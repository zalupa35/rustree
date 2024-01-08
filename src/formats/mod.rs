use crate::{app::text_processor::process_text, types::Node};
use fltk::dialog;
use std::fs;

mod html;
pub mod rtd;

fn ask_table_of_contents() -> bool {
    dialog::choice2_default("Create table of contents?", "No", "Yes", "") == Some(1)
}

pub fn node_to_text(node: Node) -> String {
    format!(
        "- {}\n{}\n\n{}",
        node.name,
        html::remove_html_tags(node.content),
        node.children
            .iter()
            .map(|n| node_to_text(n.clone()))
            .collect::<String>()
    )
}

pub fn node_to_html(node: Node, is_document: bool) -> String {
    if is_document {
        format!(
            "{}{}",
            if ask_table_of_contents() {
                generate_table_of_contents(node.clone(), is_document) + "<hr>"
            } else {
                String::new()
            },
            node_to_html(node, false)
        )
    } else {
        format!(
            "<h1 id=\"{}\">{}</h1>{}{}",
            node.id,
            node.name,
            node.content,
            node.children
                .iter()
                .map(|n| node_to_html(n.clone(), false))
                .collect::<String>()
        )
    }
}

fn generate_table_of_contents(node: Node, is_document: bool) -> String {
    if is_document {
        format!(
            "<h1>Table of Contents</h1><ul>{}</ul>",
            generate_table_of_contents(node, false)
        )
    } else {
        format!(
            "<li><a href=\"#{}\">{}</a>{}</li>",
            node.id,
            node.name,
            if !node.children.is_empty() {
                format!(
                    "<ul>{}</ul>",
                    node.children
                        .iter()
                        .map(|n| generate_table_of_contents(n.clone(), false))
                        .collect::<String>()
                )
            } else {
                String::new()
            }
        )
    }
}

pub fn node_to_md(node: Node, is_document: bool, is_html: bool) -> String {
    if is_document {
        format!(
            "{}{}",
            if ask_table_of_contents() {
                generate_table_of_contents(node.clone(), is_document) + "<hr>"
            } else {
                String::new()
            },
            node_to_md(node, false, dialog::choice2_default(
                "Convert to markdown? (it doesn't work very well) otherwise the nodes will be unchanged (in HTML format that works in GFM)",
                "No",
                "Yes",
                ""
            ) == Some(0))
        )
    } else {
        let content = process_text(node.content, |s| s.to_string(), |s| s.to_string());
        format!(
            "<h2 id=\"{}\">{}</h2>{}{}",
            node.id,
            node.name,
            if !is_html {
                html::to_markdown(content)
            } else {
                content
            },
            node.children
                .iter()
                .map(|n| node_to_md(n.clone(), false, is_html))
                .collect::<String>()
        )
    }
}

pub fn node_to_json(node: Node) -> String {
    serde_json::to_string_pretty(&node).unwrap()
}

pub fn save_format_dialog(name: String, bytes: Vec<u8>) {
    let mut nfc = dialog::NativeFileChooser::new(dialog::FileDialogType::BrowseSaveFile);
    nfc.set_filter("");
    nfc.set_preset_file(&name);
    nfc.show();
    let filename = nfc.filename();
    if !filename.to_string_lossy().is_empty() {
        if let Err(e) = fs::write(filename, bytes) {
            dialog::alert_default(&e.to_string());
        }
    }
}
