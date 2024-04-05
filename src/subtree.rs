use std::ops::Deref;

use crate::{node::NodeId, tree::Tree};

pub struct SubTree<'a, T> {
    tree: &'a mut Tree<T>,
    tree_root_id: NodeId,
    subtree_root_parent: Option<NodeId>,
    is_detached: bool,
}

impl<'a, T> SubTree<'a, T> {
    fn new_internal(tree: &'a mut Tree<T>, subtree_root_id: NodeId, is_detached: bool) -> Self {
        let tree_root_id = tree.root_id();
        let subtree_root_parent = if !is_detached {
            // if Subtree is not detached, we temporarily unset the Subtree's root parent
            // to simulate one
            let parent = tree.relatives_of(subtree_root_id).parent;
            tree.get_mut(subtree_root_id).relatives.parent = None;
            parent
        } else {
            None
        };
        // SubTree has the control over Tree until Drop
        // Tree's root_id will temporarily point to the SubTree's root_id
        // This will be reverted during Drop
        tree.root_id = Some(subtree_root_id);

        Self {
            tree,
            tree_root_id,
            subtree_root_parent,
            is_detached,
        }
    }

    pub(crate) fn new_detached(tree: &'a mut Tree<T>, subtree_root_id: NodeId) -> Self {
        Self::new_internal(tree, subtree_root_id, true)
    }

    pub(crate) fn new(tree: &'a mut Tree<T>, subtree_root_id: NodeId) -> Self {
        Self::new_internal(tree, subtree_root_id, false)
    }
}

impl<'a, T> Drop for SubTree<'a, T> {
    fn drop(&mut self) {
        // root_id is currently the detached node_id
        let detached_id = self.tree.root_id();
        if !self.is_detached {
            // restore subtree's root parent in case it was not detached
            let subtree_root_parent = self.subtree_root_parent;
            self.tree.get_mut(detached_id).relatives.parent = subtree_root_parent;
        }
        // recover the original root_id
        self.tree.root_id = Some(self.tree_root_id);
        if self.is_detached {
            // recursively delete the detached node and all children
            // if the subtree is detached
            self.tree.remove(detached_id);
        }
    }
}

impl<'a, T> Deref for SubTree<'a, T> {
    type Target = Tree<T>;

    fn deref(&self) -> &Self::Target {
        self.tree
    }
}
