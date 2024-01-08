use std::{collections::BTreeMap, path::PathBuf};

use crate::utils::*;
use rand::Rng;
use serde::{Deserialize, Serialize};

pub const RESOURCES_FILTER: &str = "Images\t*.{png,jpg,jpeg,svg,gif}";
pub const RTD_FILTER: &str = "Rustree document\t*.rtd";

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Theme {
    Light,
    Dark,
    Classic,
    Aero,
    Blue,
    AquaClassic,
    Greybird,
    HighContrast,
}

impl ToString for Theme {
    fn to_string(&self) -> String {
        match self {
            Theme::Dark => "Dark",
            Theme::Light => "Light",
            Theme::Classic => "Classic",
            Theme::Aero => "Aero",
            Theme::Blue => "Blue",
            Theme::AquaClassic => "Aqua classic",
            Theme::Greybird => "Greybird",
            Theme::HighContrast => "High contrast",
        }
        .to_string()
    }
}

impl Theme {
    pub fn get_themes() -> Vec<Theme> {
        vec![
            Theme::Light,
            Theme::Dark,
            Theme::Classic,
            Theme::Aero,
            Theme::Blue,
            Theme::AquaClassic,
            Theme::Greybird,
            Theme::HighContrast,
        ]
    }
    pub fn from_string(str: String) -> Theme {
        Theme::get_themes()
            .iter()
            .find(|e| e.to_string() == str)
            .unwrap_or(&Theme::Light)
            .clone()
    }
}

#[derive(Debug, Clone)]
pub struct Resource {
    pub bytes: Vec<u8>,
    pub id: u32,
}

impl Resource {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            id: rand::thread_rng().gen(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ResourcesManager {
    pub resources: Vec<Resource>,
    pub temp_dir: PathBuf,
}

impl ResourcesManager {
    pub fn add_resource(&mut self, res: Resource) {
        self.resources.push(res.clone());
        _ = std::fs::write(&self.temp_dir.join(res.id.to_string()), res.bytes);
    }
    pub fn clear_resources(&mut self) {
        if let Ok(d) = std::fs::read_dir(&self.temp_dir) {
            for e in d {
                if let Ok(entry) = e {
                    _ = std::fs::remove_file(entry.path());
                }
            }
        }
        self.resources.clear();
    }
    pub fn new() -> Self {
        let temp_dir = std::env::temp_dir().join("rustree_resources");
        _ = std::fs::create_dir(&temp_dir);
        Self {
            temp_dir,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone)]
pub struct Document {
    pub root_node: Node,
    pub resources: BTreeMap<String, Resource>,
}

impl Document {
    pub fn get_node(self, id: u32) -> Option<Node> {
        get_node_by_id(self.root_node, id)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Node {
    pub name: String,
    pub content: String,
    pub children: Vec<Node>,

    #[serde(skip_serializing)]
    pub id: u32,
}

impl Node {
    pub fn new(name: String, content: String, children: Vec<Node>) -> Self {
        Self {
            name,
            content,
            children,
            id: Node::generate_id(),
        }
    }
    pub fn generate_id() -> u32 {
        rand::thread_rng().gen()
    }
}

#[derive(Clone, Copy)]
pub enum TreeExportType {
    RtdNode,

    TextNode,
    TextDoc,

    HtmlNode,
    HtmlDoc,

    MdNode,
    MdDoc,

    JsonNode,
    JsonDoc,
}

#[derive(Clone, Copy)]
pub enum Message {
    ToggleEditMode,
    SelectNode(u32),

    Undo,
    Redo,

    DeleteNode,
    CreateNode,
    Quit,

    MoveNodeUp,
    MoveNodeDown,

    NodeStartedEditing,

    AddResource,
    DeleteResources,
    RenameResource,
    EditResource,

    ExportNode(TreeExportType),
    SaveAs,
    SaveDocument,
    OpenDocument,

    CopyNode,
    PasteNode,
    CutNode,

    OpenSettings,
    UpdateSettings,
    ResetShortcuts,
}
