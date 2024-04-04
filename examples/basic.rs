use std::collections::VecDeque;

use nary_tree::{node::RelativeType, tree::Tree};

fn main() {
    let mut tree = Tree::default();

    // Level 0
    let id0 = tree.set_root(0);

    // Level 1
    tree.append_child(id0, 1);
    let id2 = tree.append_child(id0, 2);
    let id4 = tree.append_child(id0, 4);

    // Level 2
    tree.append_child(id2, 3);
    tree.append_child(id4, 5);
    let id6 = tree.append_child(id4, 6);

    // Level 3
    tree.append_child(id6, 7);

    //        0
    //      / | \
    //     1  2  4
    //       /  / \
    //      3  5   6
    //              \
    //               7

    let mut node_ptr = tree.create_nodeptr(id2);

    println!("2 = {}", node_ptr.data());

    println!("0 = {}", tree.data(tree.root_id()));

    node_ptr.walk(tree.root_id());
    node_ptr.walk_to(RelativeType::FirstChild);
    println!("1 = {}", node_ptr.data());
    node_ptr.walk_to(RelativeType::NextSibling);
    println!("2 = {}", node_ptr.data());
    node_ptr.walk_to(RelativeType::FirstChild);
    println!("3 = {}", node_ptr.data());
    node_ptr
        .walk_to(RelativeType::Parent)
        .walk_to(RelativeType::Parent);
    println!("0 = {}", node_ptr.data());

    println!("Traversal depth first post order:");
    println!(
        "{:?}",
        tree.traversal_depth_first_post_order(None)
            .map(|n| tree.data(n))
            .collect::<Vec<_>>()
    );

    println!("Traversal level order:");
    // Custom traversal example: level order
    let initial_context = (
        VecDeque::from(vec![tree.root_id()]),
        VecDeque::from(vec![tree.root_id()]),
    );

    println!(
        "{:?}",
        tree.traversal_user_custom(
            initial_context,
            Box::new(|tree, context| {
                let start_level = &mut context.0;
                let stack = &mut context.1;
                if let Some(node_id) = stack.pop_front() {
                    return Some(node_id);
                }
                for node_id in start_level.iter() {
                    stack.append(&mut VecDeque::from(
                        tree.children(*node_id).collect::<Vec<_>>(),
                    ));
                }
                *start_level = stack.to_owned();
                stack.pop_front()
            })
        )
        .map(|n| tree.data(n))
        .collect::<Vec<_>>()
    );

    // find node with data = 2
    let node2_id = tree
        .traversal_level_order(None)
        .find(|n| *tree.data(*n) == 2)
        .unwrap();
    println!("{:?} == {}", node2_id, tree.data(node2_id));

    println!("Detached:");
    let detached = tree.detach(id4);
    println!("root_id = {:?}", detached.root_id());
    println!(
        "{:?}",
        detached
            .traversal_level_order(None)
            .map(|n| detached.data(n))
            .collect::<Vec<_>>()
    );
    let detached_clone = detached.clone();
    drop(detached);

    println!("Detached Clone:");
    println!(
        "{:?}",
        detached_clone
            .traversal_level_order(None)
            .map(|n| detached_clone.data(n))
            .collect::<Vec<_>>()
    );

    println!("Tree minus detached nodes:");
    println!(
        "{:?}",
        tree.traversal_level_order(None)
            .map(|n| tree.data(n))
            .collect::<Vec<_>>()
    );

    let cloned_tree = tree.clone();
    drop(tree);

    println!("Cloned Tree:");
    println!(
        "{:?}",
        cloned_tree
            .traversal_level_order(None)
            .map(|n| cloned_tree.data(n))
            .collect::<Vec<_>>()
    );

    let detached_id7 = detached_clone
        .traversal_depth_first_post_order(None)
        .find(|n| *detached_clone.data(*n) == 7)
        .unwrap();
    println!("Ancestors of 7:");
    println!(
        "{:?}",
        detached_clone
            .ancestors(detached_id7)
            .map(|n| detached_clone.data(n))
            .collect::<Vec<_>>()
    );
}
