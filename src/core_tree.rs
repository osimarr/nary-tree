use crate::NodeId;
use crate::node::Node;
use crate::slab::{self, Slab};
use snowflake::ProcessUniqueId;

///
/// A wrapper around a Slab containing Node<T> values.
///
/// Groups a collection of Node<T>s with a process unique id.
///
#[derive(Debug)]
pub(crate) struct CoreTree<T> {
    id: ProcessUniqueId,
    slab: Slab<Node<T>>,
}

impl<T> CoreTree<T> {
    pub(crate) fn new(capacity: usize) -> CoreTree<T> {
        CoreTree {
            id: ProcessUniqueId::new(),
            slab: Slab::new(capacity),
        }
    }

    pub(crate) fn capacity(&self) -> usize {
        self.slab.capacity()
    }

    pub(crate) fn insert(&mut self, data: T) -> NodeId {
        let key = self.slab.insert(Node::new(data));
        self.new_node_id(key)
    }

    pub(crate) fn remove(&mut self, node_id: NodeId) -> Option<T> {
        self.filter_by_tree_id(node_id)
            .and_then(|id| self.slab.try_remove(id.index))
            .map(|node| node.data)
    }

    pub(crate) fn get(&self, node_id: NodeId) -> Option<&Node<T>> {
        self.filter_by_tree_id(node_id)
            .and_then(|id| self.slab.get(id.index))
    }

    pub(crate) fn get_mut(&mut self, node_id: NodeId) -> Option<&mut Node<T>> {
        self.filter_by_tree_id(node_id)
            .and_then(move |id| self.slab.get_mut(id.index))
    }

    #[cfg(feature = "experimental")]
    pub(crate) fn compact(&mut self) -> usize {
        use std::collections::HashMap;

        // collect a vec of rekeyed indices
        let mut rekey_tuples = Vec::new();
        self.slab.compact(|from, to| {
            // cannot create NodeId here because 'self' can't be used inside this closure
            rekey_tuples.push((from, to));
        });

        // convert to a hashmap of from(usize)->to(NodeId) for easier rekey check
        let mut rekeys = HashMap::new();
        for (from, to) in rekey_tuples.into_iter() {
            rekeys.insert(from, self.new_node_id(to));
        }

        // fix each rekeyed node's relationships
        for (from, to) in &rekeys {
            self.fix_rekeyed_node(*from, *to, &rekeys);
        }

        self.capacity()
    }

    #[cfg(feature = "experimental")]
    fn fix_rekeyed_node(
        &mut self,
        from: usize,
        to: NodeId,
        rekeys: &std::collections::HashMap<usize, NodeId>,
    ) {
        let relatives = { self.get(to).unwrap().relatives };

        // first fix the parent relation
        if let Some(parent) = relatives.parent {
            // check if parent was rekeyed
            let parent = *rekeys.get(&parent.index.index).unwrap_or(&parent);

            // unwrap() because all data are supposed to be present
            let parent = self.get_mut(parent).unwrap();
            if parent.relatives.first_child.unwrap().index.index == from {
                parent.relatives.first_child = Some(to);
            }
            if parent.relatives.last_child.unwrap().index.index == from {
                parent.relatives.last_child = Some(to);
            }
        }

        // second fix the siblings relations
        if let Some(prev) = relatives.prev_sibling {
            // check if prev_sibling was rekeyed
            let prev = *rekeys.get(&prev.index.index).unwrap_or(&prev);

            // unwrap() because all data are supposed to be present
            let prev = self.get_mut(prev).unwrap();
            if prev.relatives.next_sibling.unwrap().index.index == from {
                prev.relatives.next_sibling = Some(to);
            }
        }
        if let Some(next) = relatives.next_sibling {
            // check if next_sibling was rekeyed
            let next = *rekeys.get(&next.index.index).unwrap_or(&next);

            // unwrap() because all data are supposed to be present
            let next = self.get_mut(next).unwrap();
            if next.relatives.prev_sibling.unwrap().index.index == from {
                next.relatives.prev_sibling = Some(to);
            }
        }

        // third fix the children relations
        let mut next_child_id = relatives.first_child;
        while let Some(child_id) = next_child_id {
            // unwrap() because the data is supposed to be present
            let child = self.get_mut(child_id).unwrap();
            child.relatives.parent = Some(to);
            next_child_id = child.relatives.next_sibling;
        }
    }

    fn new_node_id(&self, index: slab::Index) -> NodeId {
        NodeId {
            tree_id: self.id,
            index,
        }
    }

    fn filter_by_tree_id(&self, node_id: NodeId) -> Option<NodeId> {
        if node_id.tree_id != self.id {
            return None;
        }
        Some(node_id)
    }

    pub(crate) fn shrink_to_fit(&mut self) {
        self.slab.shrink_to_fit();
    }
}

#[cfg_attr(tarpaulin, skip)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capacity() {
        let capacity = 5;
        let tree = CoreTree::<i32>::new(capacity);
        assert_eq!(tree.capacity(), capacity);
    }

    #[test]
    fn insert() {
        let mut tree = CoreTree::new(0);

        let id = tree.insert(1);
        let id2 = tree.insert(3);

        assert_eq!(tree.get(id).unwrap().data, 1);
        assert_eq!(tree.get(id2).unwrap().data, 3);
    }

    #[test]
    fn remove() {
        let mut tree = CoreTree::new(0);

        let id = tree.insert(1);
        assert_eq!(tree.get(id).unwrap().data, 1);

        let one = tree.remove(id);
        assert!(one.is_some());

        let one = one.unwrap();
        assert_eq!(one, 1);
    }

    #[test]
    fn get() {
        let mut tree = CoreTree::new(0);

        let id = tree.insert(1);
        let id2 = tree.insert(3);

        assert_eq!(tree.get(id).unwrap().data, 1);
        assert_eq!(tree.get(id2).unwrap().data, 3);
    }

    #[test]
    fn get_mut() {
        let mut tree = CoreTree::new(0);

        let id = tree.insert(1);
        let id2 = tree.insert(3);

        assert_eq!(tree.get_mut(id).unwrap().data, 1);
        assert_eq!(tree.get_mut(id2).unwrap().data, 3);
    }

    #[test]
    fn get_with_bad_id() {
        let mut tree = CoreTree::new(0);
        let tree2: CoreTree<i32> = CoreTree::new(0);

        let mut id = tree.insert(1);
        id.tree_id = tree2.id; // oops, wrong tree id.

        let result = tree.get(id);

        assert!(result.is_none());
    }
}
