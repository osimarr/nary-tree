use std::collections::VecDeque;

use crate::{node::NodeId, tree::Tree};

pub struct TreeIter<'a, T, C> {
    tree: &'a Tree<T>,
    context: C,
    iterator: Box<dyn Fn(&'a Tree<T>, &mut C) -> Option<NodeId>>,
}

impl<'a, T, C> TreeIter<'a, T, C> {
    pub(crate) fn new(
        tree: &'a Tree<T>,
        context: C,
        iterator: Box<dyn Fn(&'a Tree<T>, &mut C) -> Option<NodeId>>,
    ) -> Self {
        Self {
            tree,
            context,
            iterator,
        }
    }
}

impl<'a, T, C> Iterator for TreeIter<'a, T, C> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        (self.iterator)(self.tree, &mut self.context)
    }
}

impl<T> Tree<T> {
    pub fn children<'a>(&'a self, from_node_id: NodeId) -> TreeIter<'a, T, Option<NodeId>> {
        let initial_context = self.get(from_node_id).relatives.first_child;
        TreeIter::new(
            self,
            initial_context,
            Box::new(|tree, context| {
                if let Some(child) = *context {
                    *context = tree.get(child).relatives.next_sibling;
                    Some(child)
                } else {
                    None
                }
            }),
        )
    }

    pub fn leftmost_path_to_leaf<'a>(
        &'a self,
        from_node_id: Option<NodeId>,
    ) -> TreeIter<'a, T, Option<NodeId>> {
        let initial_context = from_node_id.or(self.try_root_id());
        TreeIter::new(
            self,
            initial_context,
            Box::new(|tree, context| {
                if let Some(node_id) = *context {
                    *context = tree.get(node_id).relatives.first_child;
                    Some(node_id)
                } else {
                    None
                }
            }),
        )
    }

    pub fn rightmost_path_to_leaf<'a>(
        &'a self,
        from_node_id: Option<NodeId>,
    ) -> TreeIter<'a, T, Option<NodeId>> {
        let initial_context = from_node_id.or(self.try_root_id());
        TreeIter::new(
            self,
            initial_context,
            Box::new(|tree, context| {
                if let Some(node_id) = *context {
                    *context = tree.get(node_id).relatives.last_child;
                    Some(node_id)
                } else {
                    None
                }
            }),
        )
    }

    pub fn traversal_depth_first_post_order<'a>(
        &'a self,
        from_node_id: Option<NodeId>,
    ) -> TreeIter<'a, T, Vec<NodeId>> {
        let initial_context = self
            .leftmost_path_to_leaf(from_node_id.or(self.try_root_id()))
            .collect();
        TreeIter::new(
            self,
            initial_context,
            Box::new(|tree, context| {
                let node_id = context.pop()?;
                if let Some(next_sibling) = tree.relatives_of(node_id).next_sibling {
                    context.append(&mut tree.leftmost_path_to_leaf(Some(next_sibling)).collect());
                }
                Some(node_id)
            }),
        )
    }

    pub fn traversal_level_order<'a>(
        &'a self,
        from_node_id: Option<NodeId>,
    ) -> TreeIter<'a, T, (VecDeque<NodeId>, VecDeque<NodeId>)> {
        let mut initial_context = (
            VecDeque::with_capacity(1), // start of a level's fifo
            VecDeque::with_capacity(1), // fifo
        );
        if let Some(node_id) = from_node_id.or(self.try_root_id()) {
            initial_context.0.push_front(node_id);
            initial_context.1.push_front(node_id);
        }
        TreeIter::new(
            self,
            initial_context,
            Box::new(|tree, context| {
                let start_level = &mut context.0;
                let fifo = &mut context.1;
                if let Some(node_id) = fifo.pop_front() {
                    return Some(node_id);
                }
                for node_id in start_level.iter() {
                    fifo.append(&mut VecDeque::from(
                        tree.children(*node_id).collect::<Vec<_>>(),
                    ));
                }
                *start_level = fifo.to_owned();
                fifo.pop_front()
            }),
        )
    }

    pub fn traversal_user_custom<'a, C>(
        &'a self,
        context: C,
        iterator: Box<dyn Fn(&'a Tree<T>, &mut C) -> Option<NodeId>>,
    ) -> TreeIter<'a, T, C> {
        TreeIter::new(self, context, iterator)
    }
}
