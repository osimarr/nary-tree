use crate::node::NodeId;

#[derive(Debug)]
pub(crate) struct Slab<T> {
    pub(crate) slab: tokio_slab::Slab<T>,
}

impl<T> Slab<T> {
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            slab: tokio_slab::Slab::with_capacity(capacity),
        }
    }

    pub(crate) fn insert(&mut self, data: T) -> usize {
        self.slab.insert(data)
    }

    pub(crate) fn try_get(&self, node_id: NodeId) -> Option<&T> {
        self.slab.get(node_id.index)
    }

    pub(crate) fn try_get_mut(&mut self, node_id: NodeId) -> Option<&mut T> {
        self.slab.get_mut(node_id.index)
    }

    pub(crate) fn next_index(&self) -> usize {
        self.slab.vacant_key()
    }

    pub(crate) fn exists(&self, node_id: NodeId) -> bool {
        self.slab.contains(node_id.index)
    }
}
