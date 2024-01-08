use crate::{
    history_manager::HistoryManager,
    settings::{MainSettings, UIElement, UIElementType},
    types::*,
    ui,
};
use fltk::{
    app::{App, Receiver, Sender},
    enums::*,
    prelude::*,
    text::TextBuffer,
    *,
};
use fltk_theme::{ThemeType, WidgetTheme};
use std::{collections::BTreeMap, path::PathBuf};

mod document;
mod resources;
pub mod text_processor;

#[derive(Clone)]
pub struct Application {
    pub app: App,
    pub is_node_editing: bool,
    pub node_started_editing: bool,
    pub ui: ui::UserInterface,
    pub document: Document,
    pub current_node_id: u32,
    pub main_channel: Option<(Sender<Message>, Receiver<Message>)>,
    pub history_manager: HistoryManager,
    pub is_saved: bool,
    pub document_file_path: Option<PathBuf>,
    pub resources_manager: ResourcesManager,
    pub current_copied_node: Option<Node>,
    pub main_settings: MainSettings,
}

impl Application {
    /// Run Rustree
    pub fn run(&mut self) {
        init_ui(self);
        self.app.run().unwrap();
    }

    /// Create application from configuration
    pub fn new() -> Self {
        let app = App::default();
        let document = Document {
            root_node: Node::new("New document".to_string(), String::new(), Vec::new()),
            resources: BTreeMap::new(),
        };
        let mut application = Self {
            app,
            ui: ui::UserInterface::make_window(),
            is_node_editing: false,
            document: document.clone(),
            current_node_id: document.root_node.id,
            main_channel: None,
            history_manager: HistoryManager::new(),
            is_saved: false,
            document_file_path: None,
            node_started_editing: false,
            resources_manager: ResourcesManager::new(),
            current_copied_node: None,
            main_settings: MainSettings::new(),
        };
        application.update_window_title();
        application
    }

    /// Apply setings
    pub fn apply_settings(&mut self) {
        let settings = &self.main_settings;
        self.ui
            .main_text_editor
            .set_text_size(settings.editor_text_size);
        self.apply_theme();
    }

    /// Get widget theme
    pub fn apply_theme(&self) {
        let theme = &self.main_settings.theme;
        WidgetTheme::new(match theme {
            Theme::Dark => ThemeType::Dark,
            Theme::Light => ThemeType::Metro,
            Theme::Classic => ThemeType::Classic,
            Theme::Aero => ThemeType::Aero,
            Theme::Blue => ThemeType::Blue,
            Theme::AquaClassic => ThemeType::AquaClassic,
            Theme::Greybird => ThemeType::Greybird,
            Theme::HighContrast => ThemeType::HighContrast,
        })
        .apply();
    }

    /// Set unsaved
    pub fn set_unsaved(&mut self) {
        self.is_saved = false;
        self.update_window_title();
    }

    /// Update window title
    pub fn update_window_title(&mut self) {
        self.ui.window.set_label(&format!(
            "{}{} - Rustree",
            if self.is_node_editing || !self.is_saved {
                "*"
            } else {
                ""
            },
            if let Some(path) = &self.document_file_path {
                path.to_str().unwrap()
            } else {
                "Untitled"
            }
        ));
    }

    /// Set node view value
    pub fn set_node_view_value(&mut self, value: String) {
        let processed_text = text_processor::process_text(
            value,
            |key| {
                if let Some(res) = self.document.resources.get(&key.to_string()) {
                    let temp_dir = &self.resources_manager.temp_dir;
                    return temp_dir
                        .join(res.id.to_string())
                        .to_string_lossy()
                        .to_string();
                }
                String::new()
            },
            |s| {
                format!(
                    "node://{}",
                    text_processor::process_node_path(
                        s.to_string(),
                        self.document.clone(),
                        self.document
                            .clone()
                            .get_node(self.current_node_id)
                            .unwrap()
                    )
                )
            },
        );

        self.ui.main_text_view.set_value(&processed_text);
    }
}

/// Initialize menubar
pub fn init_menubar(app: &mut Application) {
    let menubar = &mut app.ui.main_menubar;
    let (main_sender, _) = app.main_channel.unwrap();

    let (
        mut move_node_up,
        mut move_node_down,
        mut delete_node,
        mut create_node,
        mut quit,
        mut save_as,
        mut save,
        mut open,
        mut settings_btn,
        mut undo,
        mut redo,
    ) = (
        menubar.find_item("&Edit/Move node up").unwrap(),
        menubar.find_item("&Edit/Move node down").unwrap(),
        menubar.find_item("&Edit/Delete node").unwrap(),
        menubar.find_item("&Edit/Create node").unwrap(),
        menubar.find_item("&File/Quit").unwrap(),
        menubar.find_item("&File/Save as").unwrap(),
        menubar.find_item("&File/Save").unwrap(),
        menubar.find_item("&File/Open").unwrap(),
        menubar.find_item("&File/Settings").unwrap(),
        menubar.find_item("&Edit/Undo").unwrap(),
        menubar.find_item("&Edit/Redo").unwrap(),
    );

    move_node_up.emit(main_sender, Message::MoveNodeUp);
    move_node_down.emit(main_sender, Message::MoveNodeDown);
    quit.emit(main_sender, Message::Quit);
    save_as.emit(main_sender, Message::SaveAs);
    save.emit(main_sender, Message::SaveDocument);
    open.emit(main_sender, Message::OpenDocument);
    create_node.emit(main_sender, Message::CreateNode);
    delete_node.emit(main_sender, Message::DeleteNode);
    undo.emit(main_sender, Message::Undo);
    redo.emit(main_sender, Message::Redo);

    let (
        mut export_doc_txt,
        mut export_node_txt,
        mut export_node_html,
        mut export_doc_html,
        mut export_node_md,
        mut export_doc_md,
        mut export_node_rtd,
        mut export_doc_json,
        mut export_node_json,
    ) = (
        menubar
            .find_item("&File/Export/Document/Text file")
            .unwrap(),
        menubar.find_item("&File/Export/Node/Text file").unwrap(),
        menubar.find_item("&File/Export/Node/HTML").unwrap(),
        menubar.find_item("&File/Export/Document/HTML").unwrap(),
        menubar.find_item("&File/Export/Node/Markdown").unwrap(),
        menubar.find_item("&File/Export/Document/Markdown").unwrap(),
        menubar.find_item("&File/Export/Node/Rtd").unwrap(),
        menubar.find_item("&File/Export/Document/JSON").unwrap(),
        menubar.find_item("&File/Export/Node/JSON").unwrap(),
    );

    export_doc_txt.emit(main_sender, Message::ExportNode(TreeExportType::TextDoc));
    export_node_txt.emit(main_sender, Message::ExportNode(TreeExportType::TextNode));
    export_node_html.emit(main_sender, Message::ExportNode(TreeExportType::HtmlNode));
    export_doc_html.emit(main_sender, Message::ExportNode(TreeExportType::HtmlDoc));
    export_node_md.emit(main_sender, Message::ExportNode(TreeExportType::MdNode));
    export_doc_md.emit(main_sender, Message::ExportNode(TreeExportType::MdDoc));
    export_node_rtd.emit(main_sender, Message::ExportNode(TreeExportType::RtdNode));
    export_node_json.emit(main_sender, Message::ExportNode(TreeExportType::JsonNode));
    export_doc_json.emit(main_sender, Message::ExportNode(TreeExportType::JsonDoc));

    settings_btn.emit(main_sender, Message::OpenSettings);

    let shortcuts_manager = &mut app.main_settings.shortcuts_manager;

    shortcuts_manager.register_ui_element(&mut UIElement::MenuItem(open), UIElementType::OpenFile);
    shortcuts_manager.register_ui_element(&mut UIElement::MenuItem(save), UIElementType::SaveFile);
    shortcuts_manager
        .register_ui_element(&mut UIElement::MenuItem(save_as), UIElementType::SaveAsFile);
    shortcuts_manager.register_ui_element(&mut UIElement::MenuItem(quit), UIElementType::QuitApp);
    shortcuts_manager.register_ui_element(&mut UIElement::MenuItem(undo), UIElementType::Undo);
    shortcuts_manager.register_ui_element(&mut UIElement::MenuItem(redo), UIElementType::Redo);

    shortcuts_manager.register_ui_element(
        &mut UIElement::MenuItem(create_node),
        UIElementType::CreateNodeBtn,
    );
    shortcuts_manager.register_ui_element(
        &mut UIElement::MenuItem(delete_node),
        UIElementType::DeleteNodeBtn,
    );
    shortcuts_manager.register_ui_element(
        &mut UIElement::MenuItem(move_node_up),
        UIElementType::UpNodeBtn,
    );
    shortcuts_manager.register_ui_element(
        &mut UIElement::MenuItem(move_node_down),
        UIElementType::DownNodeBtn,
    );
}

/// Initialize buttons
fn init_buttons(app: &mut Application) {
    let ui = &mut app.ui;
    let (main_sender, _) = app.main_channel.unwrap();

    let (copy, paste, cut, add_res, del_res, rename_res, edit_res) = (
        &mut ui.copy_btn,
        &mut ui.paste_btn,
        &mut ui.cut_btn,
        &mut ui.add_res_btn,
        &mut ui.del_res_btn,
        &mut ui.rename_res_btn,
        &mut ui.edit_res_btn,
    );

    copy.emit(main_sender, Message::CopyNode);
    paste.emit(main_sender, Message::PasteNode);
    cut.emit(main_sender, Message::CutNode);
    add_res.emit(main_sender, Message::AddResource);
    del_res.emit(main_sender, Message::DeleteResources);
    rename_res.emit(main_sender, Message::RenameResource);
    edit_res.emit(main_sender, Message::EditResource);

    let shortcuts_manager = &mut app.main_settings.shortcuts_manager;

    shortcuts_manager
        .register_ui_element(&mut UIElement::Button(copy.to_owned()), UIElementType::Copy);
    shortcuts_manager.register_ui_element(
        &mut UIElement::Button(paste.to_owned()),
        UIElementType::Paste,
    );
    shortcuts_manager
        .register_ui_element(&mut UIElement::Button(cut.to_owned()), UIElementType::Cut);
    shortcuts_manager.register_ui_element(
        &mut UIElement::Button(del_res.to_owned()),
        UIElementType::DelResBtn,
    );
}

/// Initialize user interface
fn init_ui(main_app: &mut Application) {
    let ch = app::channel::<Message>();
    let (main_sender, _main_receiver) = app::channel::<Message>();

    main_app.main_channel = Some(ch);

    // Preventing window from closing
    main_app.ui.window.set_callback(move |_| {
        if app::event() == Event::Close {
            main_sender.send(Message::Quit);
        }
    });

    init_menubar(main_app);
    init_buttons(main_app);

    main_app.update_document_tree(None);
    main_app.update_resources(None);

    let ui = &mut main_app.ui;

    ui.resources_tree.set_show_root(false);
    ui.resources_tree.set_select_mode(tree::TreeSelect::Multi);

    let main_text_view = &mut ui.main_text_view;
    let main_text_editor = &mut ui.main_text_editor;
    let right_main_tile_group = &mut ui.right_main_tile_group;

    main_text_editor.set_buffer(TextBuffer::default());
    main_text_view.set_text_size(25);

    ui.main_tree.set_show_root(false);

    main_text_editor.set_trigger(CallbackTrigger::Changed);
    ui.node_name_input.set_trigger(CallbackTrigger::Changed);

    main_text_editor.emit(main_sender, Message::NodeStartedEditing);
    ui.node_name_input
        .emit(main_sender, Message::NodeStartedEditing);

    right_main_tile_group.handle(move |_, e| {
        if e == Event::Push && app::event_clicks() {
            main_sender.send(Message::ToggleEditMode);
        }
        false
    });

    ui.window.show();
    main_app.apply_settings();
    main_app.event_loop();
}
