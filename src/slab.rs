use crate::node::Node;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SlabIndex {
    key: usize,
    generation: usize,
}

impl SlabIndex {
    fn new(key: usize, generation: usize) -> Self {
        Self { key, generation }
    }
}

#[derive(Debug, Default)]
pub(crate) struct Slab<T> {
    slab: tokio_slab::Slab<Node<T>>,
    generation: usize,
}

impl<T> Slab<T> {
    pub(crate) fn new() -> Self {
        Self {
            slab: tokio_slab::Slab::new(),
            generation: 0,
        }
    }

    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            slab: tokio_slab::Slab::with_capacity(capacity),
            generation: 0,
        }
    }

    pub(crate) fn generation(&self) -> usize {
        self.generation
    }

    pub(crate) fn insert(&mut self, node: Node<T>) -> SlabIndex {
        SlabIndex::new(self.slab.insert(node), self.generation())
    }

    pub(crate) fn try_get(&self, index: SlabIndex) -> Option<&Node<T>> {
        self.slab.get(index.key).and_then(|n| {
            if n.generation == index.generation {
                Some(n)
            } else {
                None
            }
        })
    }

    pub(crate) fn try_get_mut(&mut self, index: SlabIndex) -> Option<&mut Node<T>> {
        self.slab.get_mut(index.key).and_then(|n| {
            if n.generation == index.generation {
                Some(n)
            } else {
                None
            }
        })
    }

    pub(crate) fn try_remove(&mut self, index: SlabIndex) -> Option<Node<T>> {
        self.slab.try_remove(index.key).and_then(|n| {
            self.generation += 1;
            Some(n)
        })
    }

    pub(crate) fn exists(&self, index: SlabIndex) -> bool {
        self.try_get(index).is_some()
    }

    pub(crate) fn next_index(&self) -> SlabIndex {
        SlabIndex::new(self.slab.vacant_key(), self.generation())
    }
}
