use crate::{
    types::{Document, Node},
    utils::node_to_tree_vec,
};
use regex::{Captures, Regex};

pub fn process_node_path(path: String, document: Document, current_node: Node) -> u32 {
    let split = path.split(':').collect::<Vec<_>>();
    let tree = node_to_tree_vec(document.clone().root_node, Vec::new(), None);
    if split.len() >= 2 {
        let arg = split[1].trim();
        match split[0].to_lowercase().trim() {
            "sibling" => {
                if let Some(parent) = tree.iter().find(|e| {
                    document
                        .clone()
                        .get_node(e.1)
                        .unwrap()
                        .children
                        .iter()
                        .find(|n| n.id == current_node.id)
                        .is_some()
                }) {
                    if let Some(item) = document
                        .get_node(parent.1)
                        .unwrap()
                        .children
                        .iter()
                        .find(|n| n.name == arg)
                    {
                        return item.id;
                    }
                }
            }
            "path" => {
                if let Some(item) = tree.iter().find(|e| e.0 == arg) {
                    return item.1;
                }
            }
            _ => {}
        }
    }
    0
}

pub fn process_text(
    text: String,
    handle_res: impl Fn(&str) -> String,
    handle_ref: impl Fn(&str) -> String,
) -> String {
    let res_regex = Regex::new(r#"resource!\("([^"]*)"\)"#).unwrap();
    let ref_regex = Regex::new(r#"reference!\("([^"]*)"\)"#).unwrap();

    ref_regex
        .replace_all(
            &res_regex.replace_all(&text, |captures: &Captures| handle_res(&captures[1])),
            |captures: &Captures| handle_ref(&captures[1]),
        )
        .to_string()
}
