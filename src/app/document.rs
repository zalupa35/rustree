use std::{fs, path::PathBuf};

use super::Application;
use crate::{formats, types::*, utils::*};
use fltk::{prelude::*, *};

impl Application {
    /// Update document tree in UI
    pub fn update_document_tree(&mut self, node_to_select: Option<u32>) {
        let tree = &mut self.ui.main_tree;
        tree.clear();
        let tree_items = node_to_tree_vec(self.document.root_node.clone(), Vec::new(), None);
        for (k, __) in tree_items.clone() {
            tree.add(k.as_str());
        }
        tree.redraw();
        let cloned_tree_items = tree_items.clone();
        let (main_sender, _) = self.main_channel.unwrap();
        tree.set_callback(move |t| {
            if let Some(item) = t.first_selected_item() {
                let mut path = get_tree_item_path(item, Vec::new());
                path.reverse();
                if let Some(id) = cloned_tree_items.iter().find(|e| e.0 == path.join("/")) {
                    main_sender.send(Message::SelectNode(id.1));
                }
            }
        });
        if let Some(id) = node_to_select {
            for (k, v) in tree_items.clone() {
                if v == id {
                    if let Some(mut item) = tree.find_item(&k) {
                        item.select_toggle();
                        main_sender.send(Message::SelectNode(id));
                    };
                    break;
                }
            }
        } else {
            self.current_node_id = self.document.root_node.id;
            main_sender.send(Message::SelectNode(self.document.root_node.id));
        }
    }

    /// Set document file
    pub fn set_document_file(&mut self, path: PathBuf) {
        self.document_file_path = Some(path);
        self.update_window_title();
    }

    /// Save node
    pub fn save_editing_node(&mut self) -> bool {
        let need_to_save = self.node_started_editing;
        if need_to_save {
            let current_editing_node = self
                .document
                .clone()
                .get_node(self.current_node_id)
                .unwrap();
            match ask_save_dialog("Save node?") {
                SaveDialogAnswer::Cancel => return true,
                SaveDialogAnswer::Yes => {
                    let name = self.ui.node_name_input.value();
                    if !name.trim().is_empty() {
                        let id = self.clone().current_node_id;
                        let new_node = Node {
                            id: current_editing_node.clone().id,
                            name: name.clone(),
                            content: self.ui.main_text_editor.buffer().unwrap().text(),
                            children: current_editing_node.clone().children,
                        };
                        if find_node_with_same_name_in_same_parent_node(
                            self.clone().document.root_node,
                            id.clone(),
                            name.clone(),
                        )
                        .is_some()
                        {
                            self.document = self.history_manager.register_document_modification(
                                self.document.clone(),
                                DocumentModification::EditNode(id, new_node),
                            );
                            self.update_document_tree(Some(current_editing_node.clone().id));
                        } else {
                            dialog::alert_default(&format!("There is already a node named {name}"));
                            return true;
                        }
                    } else {
                        dialog::alert_default("Node name can't be empty");
                        return true;
                    }
                }
                SaveDialogAnswer::No => {}
            }
        }
        _ = &self.ui.main_text_view.show();
        _ = &self.ui.text_editor_group.hide();
        self.is_node_editing = false;
        self.node_started_editing = false;
        self.update_window_title();
        !need_to_save
    }

    /// Save document dialog
    pub fn save_document_dialog(&mut self) -> bool {
        if !self.is_saved || self.document_file_path.is_none() {
            match ask_save_dialog("Save document?") {
                SaveDialogAnswer::Cancel => return false,
                SaveDialogAnswer::Yes => return self.save_document(),
                SaveDialogAnswer::No => {}
            }
        }
        true
    }

    /// Open document
    pub fn open_document(&mut self) {
        if self.is_node_editing && self.save_editing_node() && self.node_started_editing {
            return;
        } else if self.save_document_dialog() {
            let mut nfc = dialog::NativeFileChooser::new(dialog::FileDialogType::BrowseFile);
            nfc.set_filter(RTD_FILTER);
            nfc.show();
            let filename = nfc.filename();
            if !filename.to_string_lossy().is_empty() {
                if let Ok(bytes) = fs::read(&filename) {
                    match formats::rtd::deserealize(bytes) {
                        Ok(doc) => {
                            self.document = formats::rtd::rtd_document_to_document(doc);
                            self.set_document_file(filename);
                            self.is_saved = true;
                            self.update_document_tree(None);
                            self.update_resources(None);
                            self.update_window_title();
                        }
                        Err(e) => dialog::alert_default(&e.to_string()),
                    }
                } else {
                    dialog::alert_default("Can't read file");
                }
            }
        }
    }

    /// Save document
    pub fn save_document(&mut self) -> bool {
        let document = self.document.clone();
        if let Some(file_path) = &self.document_file_path {
            if let Ok(bytes) =
                formats::rtd::serealize(formats::rtd::document_to_rtd_document(document))
            {
                if let Err(e) = fs::write(file_path, bytes) {
                    dialog::alert_default(&e.to_string());
                    return false;
                }
            }
        } else {
            let mut nfc = dialog::NativeFileChooser::new(dialog::FileDialogType::BrowseSaveFile);
            nfc.set_filter(RTD_FILTER);
            nfc.set_preset_file("Untitled.rtd");
            nfc.show();
            let filename = nfc.filename();
            if !filename.to_string_lossy().is_empty() {
                if let Ok(bytes) =
                    formats::rtd::serealize(formats::rtd::document_to_rtd_document(document))
                {
                    if let Err(e) = fs::write(filename.clone(), bytes) {
                        dialog::alert_default(&e.to_string());
                        return false;
                    } else {
                        self.set_document_file(filename);
                    }
                }
            } else {
                return false;
            }
        }
        self.is_saved = true;
        self.update_window_title();
        true
    }

    /// Save as
    pub fn save_as_btn(&mut self) {
        let mut nfc = dialog::NativeFileChooser::new(dialog::FileDialogType::BrowseSaveFile);
        nfc.set_filter(RTD_FILTER);
        nfc.set_preset_file("Untitled.rtd");
        nfc.show();
        let filename = nfc.filename();
        if !filename.to_string_lossy().is_empty() {
            if let Ok(bytes) = formats::rtd::serealize(formats::rtd::document_to_rtd_document(
                self.document.clone(),
            )) {
                if let Err(e) = fs::write(filename, bytes) {
                    dialog::alert_default(&e.to_string());
                }
            }
        }
    }

    /// Get selected node id
    pub fn get_selected_node_id(&mut self) -> Option<u32> {
        if let Some(item) = self.ui.main_tree.first_selected_item() {
            let mut path = get_tree_item_path(item, Vec::new());
            path.reverse();
            let document = self.document.clone();
            let tree = node_to_tree_vec(document.clone().root_node.clone(), Vec::new(), None);
            if let Some((_, id)) = tree.iter().find(|e| e.0 == path.join("/")) {
                return Some(*id);
            }
            return None;
        }
        None
    }

    /// Creating new node
    pub fn create_new_node(&mut self) {
        if let Some(id) = self.get_selected_node_id() {
            self.document = self.history_manager.register_document_modification(
                self.document.clone(),
                DocumentModification::CreateNode(id, Node::generate_id()),
            );
            let tree = &mut self.ui.main_tree;
            _ = tree.deselect_all(&tree.first_selected_item().unwrap(), true);
            self.update_document_tree(Some(id));
        }
    }

    /// Deleting selected node
    pub fn delete_node(&mut self) {
        if let Some(id) = self.get_selected_node_id() {
            self.document = self.history_manager.register_document_modification(
                self.document.clone(),
                DocumentModification::DeleteNode(id),
            );
            let tree = &mut self.ui.main_tree;
            _ = tree.deselect_all(&tree.first_selected_item().unwrap(), true);
            self.update_document_tree(None);
        }
    }

    /// Move node up
    pub fn move_node_up(&mut self) {
        if let Some(id) = self.get_selected_node_id() {
            self.document = self.history_manager.register_document_modification(
                self.document.clone(),
                DocumentModification::MoveNode(id, MoveDirection::Up),
            );
            let tree = &mut self.ui.main_tree;
            _ = tree.deselect_all(&tree.first_selected_item().unwrap(), true);
            self.update_document_tree(Some(id));
        }
    }

    /// Move node down
    pub fn move_node_down(&mut self) {
        if let Some(id) = self.get_selected_node_id() {
            self.document = self.history_manager.register_document_modification(
                self.document.clone(),
                DocumentModification::MoveNode(id, MoveDirection::Down),
            );
            let tree = &mut self.ui.main_tree;
            _ = tree.deselect_all(&tree.first_selected_item().unwrap(), true);
            self.update_document_tree(Some(id));
        }
    }

    /// Cut node
    pub fn cut_node(&mut self) {
        if let Some(id) = self.get_selected_node_id() {
            if let Some(node) = &self.document.clone().get_node(id) {
                self.current_copied_node = Some(randomize_node(node));
                self.document = self.history_manager.register_document_modification(
                    self.document.clone(),
                    DocumentModification::DeleteNode(id),
                );
                self.update_document_tree(None);
            }
        }
    }

    /// Copy node
    pub fn copy_node(&mut self) {
        if let Some(id) = self.get_selected_node_id() {
            if let Some(node) = &self.document.clone().get_node(id) {
                self.current_copied_node = Some(randomize_node(node));
            }
        }
    }

    /// Paste node
    pub fn paste_node(&mut self) {
        if let Some(copied) = self.clone().current_copied_node {
            if let Some(id) = self.get_selected_node_id() {
                self.document = self.history_manager.register_document_modification(
                    self.document.clone(),
                    DocumentModification::PasteNode(copied.clone(), id),
                );
                self.update_document_tree(Some(copied.id));
            }
        }
    }

    /// Undo action
    pub fn undo_action(&mut self) {
        if let Some(doc) = self.history_manager.undo_action(self.document.clone()) {
            self.document = doc;
            self.update_document_tree(Some(self.current_node_id));
        }
    }

    /// Redo action
    pub fn redo_action(&mut self) {
        if let Some(doc) = self.history_manager.redo_action(self.document.clone()) {
            self.document = doc;
            self.update_document_tree(Some(self.current_node_id));
        }
    }
}
