use std::collections::BTreeMap;

use crate::{
    app::Application,
    formats,
    settings::{self, update_shortcuts_browser, ShortcutsManager, UIElement},
    types::*,
    utils::ask_shortcut,
};
use fltk::{app, enums::Shortcut, prelude::*};

impl Application {
    pub fn event_loop(&mut self) {
        let (_, main_receiver) = self.main_channel.unwrap();
        while self.app.wait() {
            let root_node = self.document.clone().root_node;
            let current_editing_node = self
                .document
                .clone()
                .get_node(self.current_node_id)
                .unwrap_or(root_node.clone());

            let main_text_view = &mut self.ui.main_text_view;
            let filename = main_text_view.filename();
            let link_href = filename.to_str().unwrap();
            if let Some(stripped) = link_href.strip_prefix("node://") {
                _ = main_text_view.load("");
                if let Ok(id) = &stripped.parse::<u32>() {
                    self.update_document_tree(Some(
                        if self.document.clone().get_node(*id).is_some() {
                            *id
                        } else {
                            root_node.id
                        },
                    ));
                }
            }

            if let Some(msg) = main_receiver.recv() {
                match msg {
                    Message::ToggleEditMode => {
                        if !self.is_node_editing {
                            self.is_node_editing = true;

                            self.ui.main_text_view.hide();
                            self.ui
                                .main_text_editor
                                .buffer()
                                .unwrap()
                                .set_text(&current_editing_node.clone().content);
                            self.ui.text_editor_group.show();
                            self.ui
                                .node_name_input
                                .set_value(&current_editing_node.clone().name);
                            self.update_window_title();
                            self.set_unsaved();
                        } else if self.save_editing_node() {
                            continue;
                        }
                    }
                    Message::SelectNode(id) => {
                        if self.is_node_editing {
                            self.save_editing_node();
                        } else {
                            self.current_node_id = id;
                            if let Some(node) = self.document.clone().get_node(id) {
                                self.set_node_view_value(node.content);
                            }
                        }
                    }
                    Message::Quit => {
                        if self.is_node_editing
                            && self.save_editing_node()
                            && self.node_started_editing
                        {
                            continue;
                        } else if self.save_document_dialog() {
                            self.resources_manager.clear_resources();
                            self.clone().main_settings.write();
                            app::quit();
                        }
                    }
                    Message::ExportNode(export_type) => {
                        let root_node = self.document.clone().root_node;
                        let node_to_export;
                        let ext;
                        let bytes;
                        match export_type {
                            TreeExportType::TextDoc => {
                                node_to_export = Some(root_node.clone());
                                ext = ".txt";
                                bytes = formats::node_to_text(root_node).into();
                            }
                            TreeExportType::TextNode => {
                                node_to_export = Some(current_editing_node.clone());
                                ext = ".txt";
                                bytes = formats::node_to_text(current_editing_node).into();
                            }
                            TreeExportType::HtmlDoc => {
                                node_to_export = Some(root_node.clone());
                                ext = ".html";
                                bytes = formats::node_to_html(root_node.clone(), true).into();
                            }
                            TreeExportType::HtmlNode => {
                                node_to_export = Some(current_editing_node.clone());
                                ext = ".html";
                                bytes = formats::node_to_html(current_editing_node, true).into();
                            }
                            TreeExportType::MdDoc => {
                                node_to_export = Some(root_node.clone());
                                ext = ".md";
                                bytes = formats::node_to_md(root_node, true, false).into();
                            }
                            TreeExportType::MdNode => {
                                node_to_export = Some(current_editing_node.clone());
                                ext = ".md";
                                bytes =
                                    formats::node_to_md(current_editing_node, true, false).into();
                            }
                            TreeExportType::RtdNode => {
                                node_to_export = Some(current_editing_node.clone());
                                ext = ".rtd";
                                bytes = if let Ok(b) = formats::rtd::serealize(
                                    formats::rtd::document_to_rtd_document(Document {
                                        root_node: current_editing_node.clone(),
                                        resources: self.document.clone().resources,
                                    }),
                                ) {
                                    b
                                } else {
                                    Vec::new()
                                };
                            }
                            TreeExportType::JsonDoc => {
                                node_to_export = Some(root_node.clone());
                                ext = ".json";
                                bytes = formats::node_to_json(root_node).into();
                            }
                            TreeExportType::JsonNode => {
                                node_to_export = Some(current_editing_node.clone());
                                ext = ".json";
                                bytes = formats::node_to_json(current_editing_node).into();
                            }
                        }
                        if let Some(node) = node_to_export {
                            formats::save_format_dialog(node.clone().name + ext, bytes);
                        }
                    }
                    Message::Undo => {
                        self.undo_action();
                        self.set_unsaved();
                    }
                    Message::Redo => {
                        self.redo_action();
                        self.set_unsaved();
                    }
                    Message::DeleteNode => {
                        if self.is_node_editing {
                            self.save_editing_node();
                        } else {
                            self.delete_node();
                            self.set_unsaved();
                        }
                    }
                    Message::CreateNode => {
                        if self.is_node_editing {
                            self.save_editing_node();
                        } else {
                            self.create_new_node();
                            self.set_unsaved();
                        }
                    }
                    Message::MoveNodeUp => {
                        if self.is_node_editing {
                            self.save_editing_node();
                        } else {
                            self.move_node_up();
                            self.set_unsaved();
                        }
                    }
                    Message::MoveNodeDown => {
                        if self.is_node_editing {
                            self.save_editing_node();
                        } else {
                            self.move_node_down();
                            self.set_unsaved();
                        }
                    }
                    Message::CopyNode => {
                        self.copy_node();
                    }
                    Message::PasteNode => {
                        self.paste_node();
                        self.set_unsaved();
                    }
                    Message::CutNode => {
                        if self.is_node_editing {
                            self.save_editing_node();
                        } else {
                            self.cut_node();
                            self.set_unsaved();
                        }
                    }
                    Message::NodeStartedEditing => {
                        self.node_started_editing = true;
                        self.set_unsaved();
                    }
                    Message::AddResource => {
                        self.add_resource_dialog();
                    }
                    Message::DeleteResources => {
                        self.delete_resources();
                    }
                    Message::RenameResource => {
                        self.rename_resource();
                    }
                    Message::EditResource => {
                        self.edit_resource();
                    }
                    Message::SaveAs => {
                        self.save_as_btn();
                    }
                    Message::OpenDocument => {
                        self.open_document();
                    }
                    Message::SaveDocument => {
                        if self.is_node_editing && !self.save_editing_node() {
                            continue;
                        }
                        self.save_document();
                    }
                    Message::OpenSettings => {
                        settings::show(self);
                    }
                    Message::UpdateSettings => {
                        let settings = &mut self.main_settings;
                        if let Some(ui) = &mut settings.ui {
                            settings.editor_text_size = ui.editor_text_size.value() as i32;
                            if let Some(index) = ui.shortcuts_browser.selected_items().first() {
                                if let Some((_, element_type)) = settings
                                    .elements_shortcuts_browser_indexes
                                    .iter()
                                    .find(|(i, _)| i == index)
                                {
                                    let shortcut = *settings
                                        .shortcuts_manager
                                        .shortcuts
                                        .get(element_type)
                                        .unwrap_or(
                                            ShortcutsManager::default_shortcuts()
                                                .get(element_type)
                                                .unwrap_or(&Shortcut::None),
                                        );
                                    settings
                                        .shortcuts_manager
                                        .set_shortcut(element_type.clone(), ask_shortcut(shortcut));
                                    settings.elements_shortcuts_browser_indexes =
                                        update_shortcuts_browser(
                                            &mut ui.shortcuts_browser,
                                            settings.shortcuts_manager.shortcuts.clone(),
                                        );
                                }
                            }
                            if let Some(theme) = ui.theme_choice.value() {
                                settings.theme = Theme::from_string(theme);
                            }
                        }
                        for e in settings.shortcuts_manager.elements.values_mut() {
                            if let UIElement::MenuItem(item) = e {
                                item.set_shortcut(Shortcut::None);
                            }
                        }
                        self.apply_settings();
                    }
                    Message::ResetShortcuts => {
                        let settings = &mut self.main_settings;
                        if let Some(ui) = &mut settings.ui {
                            settings.shortcuts_manager.shortcuts = BTreeMap::new();
                            settings.elements_shortcuts_browser_indexes = update_shortcuts_browser(
                                &mut ui.shortcuts_browser,
                                settings.shortcuts_manager.shortcuts.clone(),
                            );
                            self.apply_settings();
                        }
                    }
                }
            }
        }
    }
}
