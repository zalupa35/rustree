use super::Application;
use crate::types::*;
use fltk::{prelude::*, *};
use std::{collections::BTreeMap, fs};

impl Application {
    /// Update resources
    pub fn update_resources(&mut self, to_select: Option<String>) {
        let tree = &mut self.ui.resources_tree;
        tree.clear();
        self.resources_manager.clear_resources();
        for (name, res) in self.document.clone().resources {
            self.resources_manager.add_resource(res);
            tree.add(&name);
        }
        // unfortunately it can't draw itself without calling redraw
        tree.redraw();
        if let Some(name) = to_select {
            if let Some(mut item) = tree.find_item(&name) {
                item.select_toggle();
            }
        }
    }

    /// Add resource dialog
    pub fn add_resource_dialog(&mut self) {
        let mut nfc = dialog::NativeFileChooser::new(dialog::FileDialogType::BrowseFile);
        nfc.set_filter(RESOURCES_FILTER);
        nfc.show();

        let filename = nfc.filename();
        if !filename.to_string_lossy().is_empty() {
            if let Ok(bytes) = fs::read(filename.clone()) {
                if let Some(res_name) = dialog::input_default(
                    "Enter resource name",
                    filename.file_name().unwrap().to_str().unwrap(),
                ) {
                    if !self.document.clone().resources.contains_key(&res_name) {
                        let res = Resource::new(bytes);
                        self.document.resources.insert(res_name.clone(), res);
                        self.update_resources(Some(res_name));
                    } else {
                        dialog::alert_default(&format!("Resource {} already exists", res_name));
                    }
                }
            }
        }
    }

    /// Delete selected resources
    pub fn delete_resources(&mut self) {
        if let Some(items) = self.ui.resources_tree.get_selected_items() {
            self.document.resources = self
                .document
                .clone()
                .resources
                .into_iter()
                .filter(|(s, _)| !items.iter().any(|i| i.label().unwrap() == *s))
                .collect::<BTreeMap<_, _>>();
            self.update_resources(None);
        }
    }

    /// Rename resource
    pub fn rename_resource(&mut self) {
        if let Some(item) = self.ui.resources_tree.first_selected_item() {
            let label = &item.label().unwrap();
            let resources = self.document.clone().resources;
            if let Some(res) = resources.get(label) {
                if let Some(res_name) = dialog::input_default("Enter resource name", label) {
                    if !resources.contains_key(&res_name) {
                        self.document.resources.remove(label);
                        self.document
                            .resources
                            .insert(res_name.clone(), res.clone());
                        self.update_resources(Some(res_name));
                    } else {
                        dialog::alert_default(&format!("Resource {} already exists", res_name));
                    }
                }
            }
        }
    }

    /// Edit resource
    pub fn edit_resource(&mut self) {
        if let Some(item) = self.ui.resources_tree.first_selected_item() {
            let label = &item.label().unwrap();
            if let Some(res) = self.document.clone().resources.get(label) {
                let mut nfc = dialog::NativeFileChooser::new(dialog::FileDialogType::BrowseFile);
                nfc.set_filter(RESOURCES_FILTER);
                nfc.show();
                let filename = nfc.filename();
                if !filename.to_string_lossy().is_empty() {
                    if let Ok(bytes) = fs::read(filename.clone()) {
                        self.document.resources.insert(
                            label.clone(),
                            Resource {
                                bytes,
                                ..res.clone()
                            },
                        );
                        self.update_resources(Some(label.to_string()));
                    }
                }
            }
        }
    }
}
