use crate::{slab::SlabIndex, tree::Tree};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId {
    pub(crate) index: SlabIndex,
}

impl NodeId {
    pub(crate) fn new(index: SlabIndex) -> Self {
        Self { index }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RelativeType {
    Parent,
    FirstChild,
    LastChild,
    PrevSibling,
    NextSibling,
}

impl RelativeType {
    pub fn get_id(&self, relatives: &Relatives) -> Option<NodeId> {
        match self {
            RelativeType::Parent => relatives.parent,
            RelativeType::FirstChild => relatives.first_child,
            RelativeType::LastChild => relatives.last_child,
            RelativeType::PrevSibling => relatives.prev_sibling,
            RelativeType::NextSibling => relatives.next_sibling,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Relatives {
    pub(crate) parent: Option<NodeId>,
    pub(crate) first_child: Option<NodeId>,
    pub(crate) last_child: Option<NodeId>,
    pub(crate) prev_sibling: Option<NodeId>,
    pub(crate) next_sibling: Option<NodeId>,
}

#[derive(Debug, Default)]
pub struct Node<T> {
    pub(crate) relatives: Relatives,
    pub(crate) data: T,
}

impl<T> Node<T> {
    pub(crate) fn new(data: T) -> Self {
        Self {
            relatives: Default::default(),
            data,
        }
    }
}

pub struct NodePtr<'a, T> {
    tree: &'a Tree<T>,
    node_id: NodeId,
}

impl<'a, T> NodePtr<'a, T> {
    pub(crate) fn new(tree: &'a Tree<T>, node_id: NodeId) -> Self {
        Self { tree, node_id }
    }

    pub fn walk(&mut self, node_id: NodeId) -> &mut NodePtr<'a, T> {
        assert!(self.tree.node_exists(node_id));
        self.node_id = node_id;
        self
    }

    pub fn try_walk_to(&mut self, relative: RelativeType) -> Option<&mut NodePtr<'a, T>> {
        let relative = relative.get_id(self.relatives())?;
        self.node_id = relative;
        Some(self)
    }

    pub fn walk_to(&mut self, relative: RelativeType) -> &mut NodePtr<'a, T> {
        self.try_walk_to(relative).unwrap()
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub fn relatives(&self) -> &Relatives {
        self.tree.relatives_of(self.node_id())
    }

    pub fn data(&self) -> &T {
        &self.tree.get(self.node_id()).data
    }
}
