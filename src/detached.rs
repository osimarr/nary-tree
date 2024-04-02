use std::ops::Deref;

use crate::{node::NodeId, tree::Tree};

pub struct DetachedTree<'a, T> {
    tree: &'a mut Tree<T>,
    tree_root_id: NodeId,
}

impl<'a, T> DetachedTree<'a, T> {
    pub(crate) fn new(tree: &'a mut Tree<T>, root_id: NodeId) -> Self {
        let tree_root_id = tree.root_id();
        // DetachedTree has control over Tree until Drop
        // Tree's root_id will temporarily point to the DetachedTree's root_id
        // This will be reverted during Drop
        tree.root_id = Some(root_id);

        Self { tree, tree_root_id }
    }
}

impl<'a, T> Drop for DetachedTree<'a, T> {
    fn drop(&mut self) {
        // root_id is currently the detached node_id
        let detached_id = self.tree.root_id();
        // recover the original root_id
        self.tree.root_id = Some(self.tree_root_id);
        // recursivelly delete the detached node and all children
        self.tree.remove(detached_id);
    }
}

impl<'a, T> Deref for DetachedTree<'a, T> {
    type Target = Tree<T>;

    fn deref(&self) -> &Self::Target {
        self.tree
    }
}
