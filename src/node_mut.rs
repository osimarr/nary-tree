use crate::{
    node::{Node, NodeId},
    tree::Tree,
    RelativeType,
};

use anyhow::anyhow;

#[derive(Debug)]
pub struct NodeMut<'a, T> {
    tree: &'a mut Tree<T>,
    node_id: NodeId,
}

impl<'a, T> NodeMut<'a, T> {
    pub(crate) fn new(tree: &'a mut Tree<T>, node_id: NodeId) -> Self {
        Self { tree, node_id }
    }

    fn do_append(&mut self, data: T) -> NodeId {
        let child_id = NodeId::new(self.tree.slab.next_index());
        let mut child = Node::new(data);
        child.relatives.parent = Some(self.node_id);

        // We will update the self relatives to reference the child, and then set relatives back altogether
        let mut relatives = self.tree.get_node(self.node_id).relatives;

        if let Some(last_child) = relatives.last_child {
            debug_assert!(
                relatives.first_child.is_some(),
                "If a node has a last_child, it must have a first_child too."
            );
            self.tree.get_node_mut(last_child).relatives.next_sibling = Some(child_id);
            child.relatives.prev_sibling = Some(last_child);
        } else {
            debug_assert!(
                relatives.first_child.is_none(),
                "If a node doesn't have a last_child, it must not have a first_child too."
            );
            relatives.first_child = Some(child_id);
        }
        relatives.last_child = Some(child_id);

        let child_index = self.tree.slab.insert(child);
        // make sure the earlier next vacant index does match the actual insert index
        debug_assert_eq!(child_index, child_id.index);

        // commit the self relatives' updates
        self.tree.get_node_mut(self.node_id).relatives = relatives;

        child_id
    }

    pub fn append(&mut self, data: T) -> &mut Self {
        self.do_append(data);
        self
    }

    pub fn append_and_walk(&mut self, data: T) -> &mut Self {
        let node_id = self.do_append(data);
        self.node_id = node_id;
        self
    }

    pub fn walk(&mut self, node_id: NodeId) -> &mut Self {
        self.try_walk(node_id)
            .ok_or(anyhow!("Error trying to walk."))
            .unwrap()
    }

    pub fn try_walk(&mut self, node_id: NodeId) -> Option<&mut Self> {
        if !self.tree.slab.exists(node_id) {
            return None;
        }
        self.node_id = node_id;
        Some(self)
    }

    pub fn walk_relative(&mut self, relative: RelativeType) -> &mut Self {
        self.try_walk_relative(relative)
            .expect("Error trying to walk to empty relative")
    }

    pub fn try_walk_relative(&mut self, relative: RelativeType) -> Option<&mut Self> {
        let node = self.tree.try_get_node(self.node_id)?;
        self.node_id = relative.get_node_id(&node.relatives)?;
        Some(self)
    }

    pub fn data(&mut self) -> &mut T {
        &mut self.tree.get_node_mut(self.node_id).data
    }
}
