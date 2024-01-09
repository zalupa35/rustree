use crate::types::{Document, Node};
use fltk::{
    app, button::ShortcutButton, dialog, enums::*, prelude::*, tree::TreeItem, window::Window,
};
use rand::Rng;

/// Creates an array of paths of all nodes
pub fn node_to_tree_vec(
    node: Node,
    nodes_vec: Vec<(String, u32)>,
    root_node: Option<String>,
) -> Vec<(String, u32)> {
    let mut new_vec = nodes_vec;
    let val = if let Some(n) = root_node {
        format!("{n}/")
    } else {
        String::new()
    };
    let name = node.name.replace('/', "\\/");
    let path = format!("{val}{}", name);
    new_vec.push((path, node.id));
    for n in node.children {
        new_vec = new_vec
            .iter()
            .chain(&node_to_tree_vec(
                n,
                Vec::new(),
                Some(format!("{}{}", val.clone(), name)),
            ))
            .cloned()
            .collect();
    }
    new_vec
}

/// Creates an array of paths of children nodes
pub fn get_tree_item_path(tree_item: TreeItem, last_vec: Vec<String>) -> Vec<String> {
    let mut result = last_vec.clone();
    if !tree_item.is_root() {
        result.push(tree_item.label().unwrap());
        if let Some(parent) = tree_item.parent() {
            result.extend(get_tree_item_path(parent, last_vec.clone()));
        }
    }
    result
}

/// Finds node by id
pub fn get_node_by_id(node: Node, id: u32) -> Option<Node> {
    if node.id == id {
        return Some(node);
    } else {
        for n in node.children {
            let ni = get_node_by_id(n, id);
            if ni.is_some() {
                return ni;
            }
        }
    }
    None
}

pub enum SaveDialogAnswer {
    Yes,
    No,
    Cancel,
}

pub fn ask_save_dialog(text: &str) -> SaveDialogAnswer {
    if let Some(answer) = dialog::choice2_default(text, "Yes", "No", "Cancel") {
        if answer == 2 {
            SaveDialogAnswer::Cancel
        } else if answer == 0 {
            SaveDialogAnswer::Yes
        } else {
            SaveDialogAnswer::No
        }
    } else {
        SaveDialogAnswer::Cancel
    }
}

#[derive(Clone, Debug)]
pub enum MoveDirection {
    Up,
    Down,
}

#[derive(Clone, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum DocumentModification {
    /// Edit node with id
    EditNode(u32, Node),
    /// Delete node with id
    DeleteNode(u32),
    /// Create node in node with id
    CreateNode(u32, u32),
    /// Move node with it up or down
    MoveNode(u32, MoveDirection),
    /// Paste node (copied node, where paste node)
    PasteNode(Node, u32),
}

/// Checks if there is a node with the same name
pub fn find_node_with_same_name_in_same_parent_node(
    node: Node,
    id: u32,
    name: String,
) -> Option<Node> {
    if node.children.iter().any(|n| n.id == id)
        && node.children.iter().any(|n| n.name == name && n.id != id)
    {
        return None;
    } else {
        for c in node.clone().children {
            find_node_with_same_name_in_same_parent_node(c, id, name.clone())?;
        }
    }
    Some(node)
}

fn modify_node(node: Node, modification: DocumentModification, root: Node) -> Node {
    match modification.clone() {
        DocumentModification::EditNode(id, edited_node) => {
            if node.id == id {
                Node {
                    name: edited_node.name,
                    content: edited_node.content,
                    ..node
                }
            } else {
                Node {
                    children: node
                        .children
                        .iter()
                        .map(|n| modify_node(n.clone(), modification.clone(), root.clone()))
                        .collect(),
                    ..node
                }
            }
        }
        DocumentModification::DeleteNode(id) => Node {
            children: node
                .children
                .iter()
                .filter(|n| n.id != id)
                .map(|n| {
                    modify_node(
                        n.clone(),
                        DocumentModification::DeleteNode(id),
                        root.clone(),
                    )
                })
                .collect(),
            ..node
        },
        DocumentModification::CreateNode(id, new_id) => {
            if node.id == id {
                Node {
                    children: node
                        .children
                        .iter()
                        .chain(vec![&Node {
                            name: format!("New node ({})", rand::thread_rng().gen::<u16>()),
                            children: Vec::new(),
                            content: String::new(),
                            id: new_id,
                        }])
                        .cloned()
                        .collect(),
                    ..node
                }
            } else {
                Node {
                    children: node
                        .children
                        .iter()
                        .map(|n| modify_node(n.clone(), modification.clone(), root.clone()))
                        .collect(),
                    ..node
                }
            }
        }
        DocumentModification::MoveNode(id, direction) => {
            if let Some(index) = node.children.clone().iter().position(|n| n.id == id) {
                let mut new_children = node.children;
                let element = new_children.remove(index);
                match direction {
                    MoveDirection::Up => {
                        if index > 0 {
                            new_children.insert(index - 1, element);
                        } else {
                            new_children.push(element);
                        }
                    }
                    MoveDirection::Down => {
                        if index < new_children.len() {
                            new_children.insert(index + 1, element);
                        } else {
                            new_children.insert(0, element);
                        }
                    }
                }
                Node {
                    children: new_children,
                    ..node
                }
            } else {
                Node {
                    children: node
                        .children
                        .iter()
                        .map(|n| modify_node(n.clone(), modification.clone(), root.clone()))
                        .collect(),
                    ..node
                }
            }
        }
        DocumentModification::PasteNode(copied, to_paste) => {
            if node.id == to_paste {
                let mut children = node.clone().children;
                let name = copied.clone().name;
                let rand_num = &rand::thread_rng().gen::<u16>().to_string();
                children.push(Node {
                    name: name.clone()
                        + if find_node_with_same_name_in_same_parent_node(
                            node.clone(),
                            copied.id,
                            name.clone(),
                        )
                        .is_some()
                        {
                            rand_num
                        } else {
                            ""
                        },
                    ..randomize_node(&copied)
                });
                Node { children, ..node }
            } else {
                Node {
                    children: node
                        .children
                        .iter()
                        .map(|n| modify_node(n.clone(), modification.clone(), root.clone()))
                        .collect(),
                    ..node
                }
            }
        }
    }
}

/// Modify document
pub fn modify_document(document: Document, modification: DocumentModification) -> Document {
    Document {
        root_node: modify_node(document.root_node.clone(), modification, document.root_node),
        ..document
    }
}

/// Get node parent
pub fn get_node_parent(node: Node, id: u32) -> Option<u32> {
    if node.id == id {
        return None;
    }
    for child in &node.children {
        if child.id == id {
            return Some(node.id);
        }
        if let Some(parent_id) = get_node_parent(child.clone(), id) {
            return Some(parent_id);
        }
    }
    None
}

/// Randomizes node id
pub fn randomize_node(node: &Node) -> Node {
    Node {
        id: Node::generate_id(),
        children: node.children.iter().map(randomize_node).collect(),
        ..node.clone()
    }
}

pub fn ask_shortcut(start_shortcut: Shortcut) -> Shortcut {
    let (s, r) = app::channel::<bool>();
    let mut clicks = 0;
    let mut win = Window::new(100, 100, 300, 200, "Shortcut");
    let mut btn = ShortcutButton::default().with_size(win.w(), win.h());
    win.make_modal(true);
    btn.set_label_size(30);
    btn.set_down_frame(fltk::enums::FrameType::NoBox);
    btn.set_label("Click to start");
    win.end();
    win.show();
    btn.handle(move |e, ev| {
        if ev == Event::Push {
            clicks += 1;
            if clicks == 2 {
                s.send(true);
            } else {
                e.window().unwrap().set_label("Click to end");
                e.set_value(start_shortcut);
            }
        }
        false
    });
    while app::wait() && win.visible() {
        if r.recv().is_some() {
            win.hide();
            return btn.value();
        }
    }
    start_shortcut
}
