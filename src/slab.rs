use crate::node::Node;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SlabIndex {
    pub(crate) key: usize,
    pub(crate) generation: usize,
}

impl SlabIndex {
    pub(crate) fn new(key: usize, generation: usize) -> Self {
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
        // try_get() does a generation check
        if self.try_get(index).is_none() {
            return None;
        }
        self.slab.try_remove(index.key).map(|n| {
            self.generation += 1;
            n
        })
    }

    pub(crate) fn exists(&self, index: SlabIndex) -> bool {
        self.try_get(index).is_some()
    }

    pub(crate) fn next_index(&self) -> SlabIndex {
        SlabIndex::new(self.slab.vacant_key(), self.generation())
    }

    pub(crate) fn capacity(&self) -> usize {
        self.slab.capacity()
    }

    pub(crate) fn len(&self) -> usize {
        self.slab.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_with_capacity() {
        let capacity = 10usize;
        let slab: Slab<i32> = Slab::with_capacity(capacity);

        assert_eq!(capacity, slab.capacity());
    }

    #[test]
    fn remove_increases_generation() {
        let mut slab = Slab::default();
        let index = slab.insert(Node::new(1, slab.generation()));
        let generation = slab.generation();
        slab.try_remove(index).unwrap();

        assert_eq!(generation + 1, slab.generation());
    }

    #[test]
    fn insert_increases_len() {
        let mut slab = Slab::default();
        assert_eq!(slab.len(), 0);
        slab.insert(Node::new(1, slab.generation()));
        assert_eq!(slab.len(), 1);
    }

    #[test]
    fn insert_and_get() {
        let mut slab = Slab::default();
        let index = slab.insert(Node::new(1, slab.generation()));
        let node = slab.try_get(index).unwrap();
        assert_eq!(node.data, 1);
    }

    #[test]
    fn insert_and_get_mut() {
        let mut slab = Slab::default();
        let index = slab.insert(Node::new(1, slab.generation()));
        let node = slab.try_get_mut(index).unwrap();
        assert_eq!(node.data, 1);
        node.data = 2;
        let node = slab.try_get_mut(index).unwrap();
        assert_eq!(node.data, 2);
    }

    #[test]
    fn insert_remove_get() {
        let mut slab = Slab::default();
        let index = slab.insert(Node::new(1, slab.generation()));
        assert!(slab.try_get(index).is_some());
        slab.try_remove(index);
        assert!(slab.try_get(index).is_none());
    }

    #[test]
    fn remove_twice() {
        let mut slab = Slab::default();
        let index = slab.insert(Node::new(1, slab.generation()));
        assert!(slab.try_get(index).is_some());
        let removed = slab.try_remove(index);
        assert!(removed.is_some());
        let removed = slab.try_remove(index);
        assert!(removed.is_none());
    }

    #[test]
    fn check_if_exists() {
        let mut slab = Slab::default();
        let index = slab.insert(Node::new(1, slab.generation()));
        assert!(slab.exists(index));
        slab.try_remove(index);
        assert!(!slab.exists(index));
    }

    #[test]
    fn get_with_wrong_generation() {
        let mut slab = Slab::default();
        let mut index = slab.insert(Node::new(1, slab.generation()));
        assert!(slab.exists(index));
        index.generation += 1;
        assert!(!slab.exists(index));
    }

    #[test]
    fn get_mut_with_wrong_generation() {
        let mut slab = Slab::default();
        let mut index = slab.insert(Node::new(1, slab.generation()));
        assert!(slab.try_get_mut(index).is_some());
        index.generation += 1;
        assert!(slab.try_get_mut(index).is_none());
    }

    #[test]
    fn remove_with_wrong_generation() {
        let mut slab = Slab::default();
        let index = slab.insert(Node::new(1, slab.generation()));
        assert!(slab.try_remove(index).is_some());
        let mut index = slab.insert(Node::new(1, slab.generation()));
        index.generation += 1;
        assert!(slab.try_remove(index).is_none());
    }
}
