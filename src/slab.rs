#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SlabIndex {
    key: usize,
}

impl SlabIndex {
    fn new(key: usize) -> Self {
        Self { key }
    }
}

#[derive(Debug, Default)]
pub(crate) struct Slab<T> {
    slab: tokio_slab::Slab<T>,
}

impl<T> Slab<T> {
    pub(crate) fn insert(&mut self, data: T) -> SlabIndex {
        SlabIndex::new(self.slab.insert(data))
    }

    pub(crate) fn try_get(&self, index: SlabIndex) -> Option<&T> {
        self.slab.get(index.key)
    }

    pub(crate) fn try_get_mut(&mut self, index: SlabIndex) -> Option<&mut T> {
        self.slab.get_mut(index.key)
    }

    pub(crate) fn try_remove(&mut self, index: SlabIndex) -> Option<T> {
        self.slab.try_remove(index.key)
    }

    pub(crate) fn exists(&self, index: SlabIndex) -> bool {
        self.slab.contains(index.key)
    }

    pub(crate) fn next_index(&self) -> SlabIndex {
        SlabIndex::new(self.slab.vacant_key())
    }
}
