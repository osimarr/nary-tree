use crate::{
    node::{Node, NodeId},
    node_mut::NodeMut,
    slab::Slab,
};

#[derive(Debug)]
pub struct Tree<T> {
    pub(crate) slab: Slab<Node<T>>,
    root_id: Option<NodeId>,
}

impl<T> Default for Tree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Tree<T> {
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            slab: Slab::with_capacity(capacity),
            root_id: None,
        }
    }

    pub fn set_root(&mut self, data: T) -> NodeId {
        let index = self.slab.insert(Node::new(data));
        let root_id = NodeId::new(index);
        self.root_id = Some(root_id);
        root_id
    }

    pub fn root_id(&self) -> NodeId {
        self.try_root_id().expect("Root was not set")
    }

    pub fn try_root_id(&self) -> Option<NodeId> {
        self.root_id
    }

    // pub fn try_root(&self) -> Option<&Node<T>> {
    //     self.root_id.map(|root_id| self.try_get_node(root_id))?
    // }

    // pub fn root(&self) -> &Node<T> {
    //     self.try_root().expect("Root was not set")
    // }

    pub fn try_root_mut(&mut self) -> Option<NodeMut<T>> {
        self.root_id.map(|root_id| NodeMut::new(self, root_id))
    }

    pub fn root_mut(&mut self) -> NodeMut<T> {
        self.try_root_mut().expect("Root was not set")
    }

    pub(crate) fn get_node_mut(&mut self, node_id: NodeId) -> &mut Node<T> {
        self.try_get_node_mut(node_id)
            .expect("NodeId was not found")
    }

    pub(crate) fn try_get_node_mut(&mut self, node_id: NodeId) -> Option<&mut Node<T>> {
        self.slab.try_get_mut(node_id)
    }

    pub(crate) fn get_node(&self, node_id: NodeId) -> &Node<T> {
        self.try_get_node(node_id).expect("NodeId was not found")
    }

    pub(crate) fn try_get_node(&self, node_id: NodeId) -> Option<&Node<T>> {
        self.slab.try_get(node_id)
    }
}
