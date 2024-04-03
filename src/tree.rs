use std::collections::HashMap;

use crate::{
    detached::DetachedTree,
    node::{Node, NodeId, NodePtr, Relatives},
    slab::Slab,
};

#[derive(Debug)]
pub struct Tree<T> {
    slab: Slab<Node<T>>,
    pub(crate) root_id: Option<NodeId>,
}

impl<T> Default for Tree<T> {
    fn default() -> Self {
        Self {
            slab: Slab::new(),
            root_id: Default::default(),
        }
    }
}

impl<T> Tree<T> {
    pub fn node_exists(&self, node_id: NodeId) -> bool {
        self.slab.exists(node_id.index)
    }

    pub(crate) fn try_get(&self, node_id: NodeId) -> Option<&Node<T>> {
        self.slab.try_get(node_id.index)
    }

    pub(crate) fn get(&self, node_id: NodeId) -> &Node<T> {
        self.try_get(node_id).unwrap()
    }

    pub(crate) fn try_get_mut(&mut self, node_id: NodeId) -> Option<&mut Node<T>> {
        self.slab.try_get_mut(node_id.index)
    }

    pub(crate) fn get_mut(&mut self, node_id: NodeId) -> &mut Node<T> {
        self.try_get_mut(node_id).unwrap()
    }

    pub fn data(&self, node_id: NodeId) -> &T {
        &self.get(node_id).data
    }

    pub fn data_mut(&mut self, node_id: NodeId) -> &mut T {
        &mut self.get_mut(node_id).data
    }

    pub fn create_nodeptr<'a>(&'a self, node_id: NodeId) -> NodePtr<'a, T> {
        assert!(self.node_exists(node_id));
        NodePtr::new(self, node_id)
    }

    pub fn try_relatives_of(&self, node_id: NodeId) -> Option<&Relatives> {
        self.try_get(node_id).map(|node| &node.relatives)
    }

    pub fn relatives_of(&self, node_id: NodeId) -> &Relatives {
        self.try_relatives_of(node_id).unwrap()
    }

    pub fn set_root(&mut self, data: T) -> NodeId {
        assert!(self.root_id.is_none());
        let root_id = NodeId::new(self.slab.insert(Node::new(data)));
        self.root_id = Some(root_id);
        root_id
    }

    pub fn try_root_id(&self) -> Option<NodeId> {
        self.root_id
    }

    pub fn root_id(&self) -> NodeId {
        self.try_root_id().unwrap()
    }

    pub fn append_child(&mut self, parent_id: NodeId, data: T) -> NodeId {
        assert!(self.node_exists(parent_id));
        let mut parent_relatives = *self.relatives_of(parent_id);
        let new_child_id = NodeId::new(self.slab.next_index());
        let mut new_child = Node::new(data);
        new_child.relatives.parent = Some(parent_id);

        if let Some(last_child_id) = parent_relatives.last_child {
            assert!(parent_relatives.first_child.is_some());
            self.get_mut(last_child_id).relatives.next_sibling = Some(new_child_id);
            new_child.relatives.prev_sibling = Some(last_child_id);
        } else {
            assert!(parent_relatives.first_child.is_none());
            parent_relatives.first_child = Some(new_child_id);
        }
        parent_relatives.last_child = Some(new_child_id);
        self.get_mut(parent_id).relatives = parent_relatives;

        let ret_id = NodeId::new(self.slab.insert(new_child));
        assert_eq!(ret_id, new_child_id);

        new_child_id
    }

    pub fn prepend_child(&mut self, parent_id: NodeId, data: T) -> NodeId {
        assert!(self.node_exists(parent_id));
        let mut parent_relatives = *self.relatives_of(parent_id);
        let new_child_id = NodeId::new(self.slab.next_index());
        let mut new_child = Node::new(data);
        new_child.relatives.parent = Some(parent_id);

        if let Some(first_child_id) = parent_relatives.first_child {
            assert!(parent_relatives.last_child.is_some());
            self.get_mut(first_child_id).relatives.prev_sibling = Some(new_child_id);
            new_child.relatives.next_sibling = Some(first_child_id);
        } else {
            assert!(parent_relatives.last_child.is_none());
            parent_relatives.last_child = Some(new_child_id);
        }
        parent_relatives.first_child = Some(new_child_id);
        self.get_mut(parent_id).relatives = parent_relatives;

        let ret_id = NodeId::new(self.slab.insert(new_child));
        assert_eq!(ret_id, new_child_id);

        new_child_id
    }

    pub fn is_leaf(&self, node_id: NodeId) -> bool {
        debug_assert_eq!(
            self.get(node_id).relatives.first_child.is_none(),
            self.get(node_id).relatives.last_child.is_none()
        );
        self.get(node_id).relatives.first_child.is_none()
    }

    pub fn children_len(&self, node_id: NodeId) -> usize {
        let mut len = 0usize;
        for _ in self.children(node_id) {
            len += 1;
        }
        len
    }

    pub fn try_remove(&mut self, node_id: NodeId) -> Option<T> {
        // leaves and higher level nodes first
        // node_id must be the last one
        let deletion_order = self
            .traversal_depth_first_post_order(Some(node_id))
            .collect::<Vec<_>>();

        // if deletion_order is empty, it means node_id didn't exist
        // the function will return None
        let mut data = None;
        for node_id in deletion_order {
            data = self.slab.try_remove(node_id.index);
            // all node_ids must exist
            debug_assert!(data.is_some());
        }

        data.map(|n| n.data)
    }

    pub fn remove(&mut self, node_id: NodeId) -> T {
        self.try_remove(node_id).unwrap()
    }

    fn try_detach_node<'a>(&'a mut self, node_id: NodeId) -> Option<NodeId> {
        let relatives = *self.try_relatives_of(node_id)?;

        let mut parent_relative = if let Some(parent) = relatives.parent {
            *self.relatives_of(parent)
        } else {
            // node_id is root_id, there's no parent or siblings
            debug_assert_eq!(node_id, self.root_id());
            self.root_id = None;
            return Some(node_id);
        };

        if let Some(prev_sibling) = relatives.prev_sibling {
            self.get_mut(prev_sibling).relatives.next_sibling = relatives.next_sibling;
        } else {
            parent_relative.first_child = relatives.next_sibling;
        }
        if let Some(next_sibling) = relatives.next_sibling {
            self.get_mut(next_sibling).relatives.prev_sibling = relatives.prev_sibling;
        } else {
            parent_relative.last_child = relatives.prev_sibling;
        }

        // sanity check to ensure first and last children are both either Some or None
        debug_assert_eq!(
            parent_relative.first_child.is_some(),
            parent_relative.last_child.is_some()
        );

        Some(node_id)
    }

    pub fn try_detach<'a>(&'a mut self, node_id: NodeId) -> Option<DetachedTree<'a, T>> {
        let detached_id = self.try_detach_node(node_id)?;
        Some(DetachedTree::new(self, detached_id))
    }

    pub fn detach<'a>(&'a mut self, node_id: NodeId) -> DetachedTree<'a, T> {
        self.try_detach(node_id).unwrap()
    }
}

impl<T: Clone> Tree<T> {
    pub fn clone_from_node(&self, from_node_id: Option<NodeId>) -> Tree<T> {
        let mut new_tree = Tree::default();
        let from_node_id = from_node_id.or(self.try_root_id());

        // cloned is a HashMap to translate node_id -> new_node_id
        let mut cloned = if let Some(from_node_id) = from_node_id {
            let new_root_id = new_tree.set_root(self.get(from_node_id).data.clone());
            let mut cloned = HashMap::new();
            cloned.insert(from_node_id, new_root_id);
            cloned
        } else {
            // if Tree is empty, new Tree is empty too
            return new_tree;
        };

        // parents are responsible for cloning their children and need to be cloned first
        // traversal level order will guarantee this restriction
        for parent_id in self.traversal_level_order(from_node_id) {
            let new_parent_id = *cloned.get(&parent_id).unwrap();
            for child_id in self.children(parent_id) {
                let new_child_id =
                    new_tree.append_child(new_parent_id, self.data(child_id).clone());
                cloned.insert(child_id, new_child_id);
            }
        }

        new_tree
    }
}

impl<T: Clone> Clone for Tree<T> {
    fn clone(&self) -> Self {
        self.clone_from_node(None)
    }
}
