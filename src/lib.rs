use node::{NodeId, Relatives};

extern crate slab as tokio_slab;

pub mod node_mut;
pub mod tree;

mod node;
mod slab;

#[derive(Debug)]
pub enum RelativeType {
    Parent,
    FirstChild,
    LastChild,
    NextSibling,
    PrevSibling,
}

impl RelativeType {
    pub(crate) fn get_node_id(&self, relatives: &Relatives) -> Option<NodeId> {
        match self {
            RelativeType::Parent => relatives.parent,
            RelativeType::FirstChild => relatives.first_child,
            RelativeType::LastChild => relatives.last_child,
            RelativeType::NextSibling => relatives.next_sibling,
            RelativeType::PrevSibling => relatives.prev_sibling,
        }
    }
}
