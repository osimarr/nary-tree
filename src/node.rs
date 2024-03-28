#[derive(Debug, Clone, Copy)]
pub struct NodeId {
    pub(crate) index: usize,
}

impl NodeId {
    pub(crate) fn new(index: usize) -> Self {
        Self { index }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct Relatives {
    pub(crate) parent: Option<NodeId>,
    pub(crate) first_child: Option<NodeId>,
    pub(crate) last_child: Option<NodeId>,
    pub(crate) next_sibling: Option<NodeId>,
    pub(crate) prev_sibling: Option<NodeId>,
}

#[derive(Debug)]
pub struct Node<T> {
    pub(crate) data: T,
    pub(crate) relatives: Relatives,
}

impl<T> Node<T> {
    pub(crate) fn new(data: T) -> Self {
        Self {
            data,
            relatives: Relatives::default(),
        }
    }
}
