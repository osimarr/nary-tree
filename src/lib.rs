extern crate slab as tokio_slab;

use anyhow::anyhow;

#[derive(Debug, Clone, Copy)]
pub struct NodeId {
    index: usize,
}

impl NodeId {
    pub(crate) fn new(index: usize) -> Self {
        Self { index }
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Relatives {
    pub(crate) parent: Option<NodeId>,
    pub(crate) first_child: Option<NodeId>,
    pub(crate) last_child: Option<NodeId>,
    pub(crate) next_sibling: Option<NodeId>,
    pub(crate) prev_sibling: Option<NodeId>,
}

#[derive(Debug)]
pub struct Node<T> {
    data: T,
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

#[derive(Debug)]
pub struct NodeMut<'a, T> {
    tree: &'a mut Tree<T>,
    node_id: NodeId,
}

impl<'a, T> NodeMut<'a, T> {
    pub(crate) fn new(tree: &'a mut Tree<T>, node_id: NodeId) -> Self {
        Self { tree, node_id }
    }

    fn do_append(&mut self, data: T) -> NodeId {
        let child_id = NodeId::new(self.tree.slab.next_index());
        let mut child = Node::new(data);
        child.relatives.parent = Some(self.node_id);

        // We will update the self relatives to reference the child, and then set relatives back altogether
        let mut relatives = self.tree.get_node(self.node_id).relatives;

        if let Some(last_child) = relatives.last_child {
            debug_assert!(
                relatives.first_child.is_some(),
                "If a node has a last_child, it must have a first_child too."
            );
            self.tree.get_node_mut(last_child).relatives.next_sibling = Some(child_id);
            child.relatives.prev_sibling = Some(last_child);
        } else {
            debug_assert!(
                relatives.first_child.is_none(),
                "If a node doesn't have a last_child, it must not have a first_child too."
            );
            relatives.first_child = Some(child_id);
        }
        relatives.last_child = Some(child_id);

        let child_index = self.tree.slab.insert(child);
        // make sure the earlier next vacant index does match the actual insert index
        debug_assert_eq!(child_index, child_id.index);

        // commit the self relatives' updates
        self.tree.get_node_mut(self.node_id).relatives = relatives;

        child_id
    }

    pub fn append(&mut self, data: T) -> &mut Self {
        self.do_append(data);
        self
    }

    pub fn append_and_walk(&mut self, data: T) -> &mut Self {
        let node_id = self.do_append(data);
        self.node_id = node_id;
        self
    }

    pub fn walk(&mut self, node_id: NodeId) {
        self.try_walk(node_id)
            .ok_or(anyhow!("Error trying to walk."))
            .unwrap();
    }

    pub fn try_walk(&mut self, node_id: NodeId) -> Option<NodeId> {
        if !self.tree.slab.exists(node_id) {
            return None;
        }
        self.node_id = node_id;
        Some(node_id)
    }
}

#[derive(Debug)]
pub struct Tree<T> {
    slab: Slab<Node<T>>,
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

    pub(crate) fn get_node_mut(&mut self, node_id: NodeId) -> &mut Node<T> {
        self.try_get_node_mut(node_id)
            .expect("NodeId was not found")
    }

    pub(crate) fn try_get_node_mut(&mut self, node_id: NodeId) -> Option<&mut Node<T>> {
        self.slab.try_get_mut(node_id)
    }

    pub(crate) fn get_node(&mut self, node_id: NodeId) -> &Node<T> {
        self.try_get_node(node_id).expect("NodeId was not found")
    }

    pub(crate) fn try_get_node(&mut self, node_id: NodeId) -> Option<&Node<T>> {
        self.slab.try_get(node_id)
    }
}

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

    pub(crate) fn try_get(&mut self, node_id: NodeId) -> Option<&T> {
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
