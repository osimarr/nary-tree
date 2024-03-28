use nary_tree::{tree::Tree, RelativeType};

fn main() {
    let mut tree = Tree::default();
    tree.set_root(0);
    let mut node_mut = tree.root_mut();
    node_mut.append(1).append_and_walk(2).append(3);

    println!("2 = {}", node_mut.data());

    let mut node_mut = tree.root_mut();

    println!("0 = {}", node_mut.data());

    node_mut.walk_relative(RelativeType::FirstChild);
    println!("1 = {}", node_mut.data());
    node_mut.walk_relative(RelativeType::NextSibling);
    println!("2 = {}", node_mut.data());
    node_mut.walk_relative(RelativeType::FirstChild);
    println!("3 = {}", node_mut.data());
    node_mut
        .walk_relative(RelativeType::Parent)
        .walk_relative(RelativeType::Parent);
    println!("0 = {}", node_mut.data());
}
