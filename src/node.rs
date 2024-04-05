use uuid::Uuid;

use crate::{slab::SlabIndex, tree::Tree};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId {
    pub(crate) index: SlabIndex,
    pub(crate) tree_uuid: Uuid,
}

impl NodeId {
    pub(crate) fn new(index: SlabIndex, tree_uuid: Uuid) -> Self {
        Self { index, tree_uuid }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
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
    pub(crate) generation: usize,
}

impl<T> Node<T> {
    pub(crate) fn new(data: T, generation: usize) -> Self {
        Self {
            relatives: Default::default(),
            data,
            generation,
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

#[cfg(test)]
mod tests {
    use uuid::uuid;

    use crate::{node::RelativeType, slab::SlabIndex, tree::Tree};

    use super::{NodeId, Relatives};

    #[rustfmt::skip]
    #[test]
    fn relative_type_get_id() {
        let tree_uuid = uuid!("00000000-0000-0000-0000-ffff00000000");
        let parend_id = Some(NodeId::new(SlabIndex::new(0, 0), tree_uuid));
        let first_child_id = Some(NodeId::new(SlabIndex::new(1, 0), tree_uuid));
        let last_child_id = Some(NodeId::new(SlabIndex::new(2, 0), tree_uuid));
        let prev_sibling_id = Some(NodeId::new(SlabIndex::new(3, 0), tree_uuid));
        let next_sibling_id = Some(NodeId::new(SlabIndex::new(4, 0), tree_uuid));
        let relatives = Relatives {
            parent: parend_id,
            first_child: first_child_id,
            last_child: last_child_id,
            prev_sibling: prev_sibling_id,
            next_sibling: next_sibling_id,
        };
        assert_eq!(RelativeType::Parent.get_id(&relatives), parend_id);
        assert_eq!(RelativeType::FirstChild.get_id(&relatives), first_child_id);
        assert_eq!(RelativeType::LastChild.get_id(&relatives), last_child_id);
        assert_eq!(RelativeType::PrevSibling.get_id(&relatives), prev_sibling_id);
        assert_eq!(RelativeType::NextSibling.get_id(&relatives), next_sibling_id);
    }

    #[test]
    fn nodeptr_node_id() {
        let mut tree = Tree::default();
        let root_id = tree.set_root(0);
        let nodeptr = tree.create_nodeptr(root_id);
        assert_eq!(nodeptr.node_id(), root_id);
    }

    #[test]
    fn nodeptr_walk() {
        let mut tree = Tree::default();
        let root_id = tree.set_root(0);
        let first_child_id = tree.append_child(root_id, 1);
        let mut nodeptr = tree.create_nodeptr(root_id);
        assert_eq!(nodeptr.node_id(), root_id);
        nodeptr.walk(first_child_id);
        assert_eq!(nodeptr.node_id(), first_child_id);
    }

    #[test]
    fn nodeptr_walk_to_relatives() {
        let mut tree = Tree::default();
        let root_id = tree.set_root(0);
        let first_child_id = tree.append_child(root_id, 1);
        let last_child_id = tree.append_child(root_id, 2);
        let mut nodeptr = tree.create_nodeptr(root_id);
        assert_eq!(nodeptr.node_id(), root_id);
        nodeptr.walk_to(RelativeType::FirstChild);
        assert_eq!(nodeptr.node_id(), first_child_id);
        nodeptr.walk_to(RelativeType::Parent);
        assert_eq!(nodeptr.node_id(), root_id);
        nodeptr.walk_to(RelativeType::LastChild);
        assert_eq!(nodeptr.node_id(), last_child_id);
        nodeptr.walk_to(RelativeType::PrevSibling);
        assert_eq!(nodeptr.node_id(), first_child_id);
        nodeptr.walk_to(RelativeType::NextSibling);
        assert_eq!(nodeptr.node_id(), last_child_id);
    }

    #[test]
    fn nodeptr_data() {
        let mut tree = Tree::default();
        let root = 0;
        let first_child = 1;
        let last_child = 2;
        let root_id = tree.set_root(root);
        tree.append_child(root_id, first_child);
        tree.append_child(root_id, last_child);
        let mut nodeptr = tree.create_nodeptr(root_id);
        assert_eq!(nodeptr.data(), &root);
        nodeptr.walk_to(RelativeType::FirstChild);
        assert_eq!(nodeptr.data(), &first_child);
        nodeptr.walk_to(RelativeType::Parent);
        assert_eq!(nodeptr.data(), &root);
        nodeptr.walk_to(RelativeType::LastChild);
        assert_eq!(nodeptr.data(), &last_child);
        nodeptr.walk_to(RelativeType::PrevSibling);
        assert_eq!(nodeptr.data(), &first_child);
        nodeptr.walk_to(RelativeType::NextSibling);
        assert_eq!(nodeptr.data(), &last_child);
    }

    #[test]
    fn nodeptr_relatives() {
        let mut tree = Tree::default();
        let root_id = tree.set_root(0);
        let relatives = *tree.relatives_of(root_id);
        let nodeptr = tree.create_nodeptr(root_id);
        assert_eq!(nodeptr.relatives(), &relatives);
    }
}
