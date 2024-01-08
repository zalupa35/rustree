use crate::types::{self, *};
use binrw::NullString;
use flate2::{write::*, Compression};
use std::io::prelude::*;

mod rtd_format {
    use binrw::{binrw, NullString};
    #[binrw]
    #[br(big)]
    #[derive(Debug, Clone)]
    pub struct Document {
        pub root_node: Node,
        pub resources_count: i32,
        #[br(count = resources_count - 1)]
        pub resources: Vec<Resource>,
    }
    #[binrw]
    #[derive(Debug, Clone)]
    pub struct Resource {
        pub name: NullString,
        pub bytes_length: i32,
        #[br(count = bytes_length - 1)]
        pub bytes: Vec<u8>,
    }
    #[binrw]
    #[derive(Debug, Clone)]
    pub struct Node {
        pub name: NullString,
        pub content: NullString,
        pub children_count: i32,
        #[br(count = children_count - 1)]
        pub children: Vec<Node>,
    }
}

use binrw::{BinRead, BinWrite};
use std::io::Cursor;

pub fn serealize(mut document: rtd_format::Document) -> Result<Vec<u8>, binrw::Error> {
    let mut resources = Vec::new();
    for res in document.clone().resources {
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        if let Ok(()) = encoder.write_all(&res.bytes) {
            if let Ok(bytes) = encoder.finish() {
                resources.push(rtd_format::Resource {
                    bytes_length: (bytes.len() + 1) as i32,
                    bytes,
                    ..res
                });
            } else {
                resources.push(res);
            }
        } else {
            resources.push(res);
        }
    }
    document.resources = resources;
    let mut writer = Cursor::new(Vec::new());
    document.write_be(&mut writer)?;
    Ok(writer.into_inner())
}

pub fn deserealize(bytes: Vec<u8>) -> Result<rtd_format::Document, binrw::Error> {
    let doc_result = rtd_format::Document::read(&mut Cursor::new(bytes));
    if let Ok(mut doc) = doc_result {
        let mut resources = Vec::new();
        for res in doc.resources {
            let mut decoder = DeflateDecoder::new(Vec::new());
            if let Ok(()) = decoder.write_all(&res.bytes) {
                if let Ok(bytes) = decoder.finish() {
                    resources.push(rtd_format::Resource {
                        bytes_length: (bytes.len() + 1) as i32,
                        bytes,
                        ..res
                    });
                } else {
                    resources.push(res);
                }
            } else {
                resources.push(res);
            }
        }
        doc.resources = resources;
        Ok(doc)
    } else {
        doc_result
    }
}

pub fn document_to_rtd_document(document: Document) -> rtd_format::Document {
    rtd_format::Document {
        root_node: node_to_rtd_node(&document.root_node),
        resources_count: (document.resources.len() + 1) as i32,
        resources: document
            .resources
            .iter()
            .map(|(n, res)| rtd_format::Resource {
                name: NullString::from(n.clone()),
                bytes_length: (res.bytes.len() + 1) as i32,
                bytes: res.bytes.clone(),
            })
            .collect(),
    }
}

fn node_to_rtd_node(node: &Node) -> rtd_format::Node {
    rtd_format::Node {
        name: NullString::from(node.name.clone()),
        content: NullString::from(node.content.clone()),
        children_count: (node.children.len() + 1) as i32,
        children: node.children.iter().map(node_to_rtd_node).collect(),
    }
}

pub fn rtd_document_to_document(document: rtd_format::Document) -> Document {
    Document {
        root_node: rtd_node_to_node(&document.root_node),
        resources: document
            .resources
            .into_iter()
            .map(|e| (e.name.to_string(), types::Resource::new(e.bytes)))
            .collect(),
    }
}

fn rtd_node_to_node(node: &rtd_format::Node) -> Node {
    Node::new(
        node.name.to_string(),
        node.content.to_string(),
        node.children.iter().map(rtd_node_to_node).collect(),
    )
}
