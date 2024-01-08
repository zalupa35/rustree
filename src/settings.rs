use crate::{
    app::Application,
    types::{Message, Theme},
    ui::SettingsInterface,
};
use fltk::{
    browser::HoldBrowser, button::Button, dialog, draw::shortcut_label, enums::*, menu::MenuItem,
    prelude::*,
};

use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::PathBuf};

const SETTINGS_PATH: &str = "./Rustree.ron";
const DEFAULT_EDITOR_TEXT_SIZE: i32 = 20;
const DEFAULT_THEME: Theme = Theme::Light;

/// UI element type
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Serialize)]
pub enum UIElementType {
    OpenFile,
    SaveFile,
    SaveAsFile,
    QuitApp,

    Copy,
    Paste,
    Cut,
    Undo,
    Redo,

    DeleteNodeBtn,
    CreateNodeBtn,
    UpNodeBtn,
    DownNodeBtn,

    DelResBtn,
}

impl ToString for UIElementType {
    fn to_string(&self) -> String {
        return format!("{:?}", self);
    }
}

/// UI element
#[derive(Clone, Debug)]
pub enum UIElement {
    MenuItem(MenuItem),
    Button(Button),
}

/// Settings document that can be serealized or deserealized
#[derive(Debug, Deserialize, Serialize, Clone)]
struct SettingsDocument {
    shortcuts: BTreeMap<UIElementType, i32>,
    theme: Theme,
    editor_text_size: i32,
}

impl SettingsDocument {
    fn new() -> Self {
        Self {
            shortcuts: BTreeMap::new(),
            theme: DEFAULT_THEME,
            editor_text_size: DEFAULT_EDITOR_TEXT_SIZE,
        }
    }

    fn write(self) -> Result<(), String> {
        match ron::to_string(&self) {
            Ok(s) => {
                if let Err(e) = fs::write(SETTINGS_PATH, s) {
                    Err(e.to_string())
                } else {
                    Ok(())
                }
            }
            Err(e) => Err(e.to_string()),
        }
    }
}

fn settings_document_to_settings(doc: SettingsDocument) -> MainSettings {
    MainSettings {
        theme: doc.theme,
        ui: None,
        elements_shortcuts_browser_indexes: Vec::new(),
        current_shortcut_ui_element_type: None,
        editor_text_size: doc.editor_text_size,
        shortcuts_manager: ShortcutsManager {
            shortcuts: doc
                .shortcuts
                .iter()
                .map(|(e, s)| (e.clone(), Shortcut::from_i32(*s)))
                .collect(),
            elements: BTreeMap::new(),
        },
    }
}

fn settings_to_document_settings(settings: MainSettings) -> SettingsDocument {
    SettingsDocument {
        shortcuts: settings
            .shortcuts_manager
            .shortcuts
            .iter()
            .map(|(e1, e2)| (e1.clone(), e2.bits()))
            .collect::<BTreeMap<_, _>>(),
        theme: settings.theme,
        editor_text_size: settings.editor_text_size,
    }
}

#[derive(Clone)]
pub struct MainSettings {
    pub shortcuts_manager: ShortcutsManager,
    pub theme: Theme,
    pub ui: Option<SettingsInterface>,
    pub elements_shortcuts_browser_indexes: Vec<(i32, UIElementType)>,
    pub current_shortcut_ui_element_type: Option<UIElementType>,
    pub editor_text_size: i32,
}

impl MainSettings {
    pub fn new() -> Self {
        let settings_path = PathBuf::from(SETTINGS_PATH);
        if settings_path.exists() {
            match fs::read_to_string(SETTINGS_PATH) {
                Ok(s) => match ron::from_str::<SettingsDocument>(&s) {
                    Ok(settings) => {
                        return settings_document_to_settings(settings);
                    }
                    Err(e) => {
                        dialog::alert_default(&format!(
                            "Can't parse settings:\n{}",
                            e.to_string().split(",").collect::<Vec<_>>().join("\n")
                        ));
                    }
                },
                Err(e) => {
                    dialog::alert_default(&format!("Can't read settings:\n{}", e.to_string()));
                }
            }
        } else {
            if let Err(s) = SettingsDocument::new().write() {
                dialog::alert_default(&s);
            };
        }
        Self {
            shortcuts_manager: ShortcutsManager::new(),
            theme: DEFAULT_THEME,
            ui: None,
            elements_shortcuts_browser_indexes: Vec::new(),
            current_shortcut_ui_element_type: None,
            editor_text_size: DEFAULT_EDITOR_TEXT_SIZE,
        }
    }
    pub fn write(self) {
        settings_to_document_settings(self).write().unwrap();
    }
}

pub fn update_shortcuts_browser(
    browser: &mut HoldBrowser,
    shortcuts: BTreeMap<UIElementType, Shortcut>,
) -> Vec<(i32, UIElementType)> {
    browser.clear();
    browser.add("UI element\tShortcut");
    let default_shortcuts = ShortcutsManager::default_shortcuts();
    let mut res = Vec::new();
    let mut index = 1;
    for (element_type, shortcut) in default_shortcuts {
        let shortcut = *shortcuts.get(&element_type).unwrap_or(&shortcut);
        browser.add(&format!(
            "{}\t{}",
            element_type.to_string(),
            shortcut_label(shortcut)
        ));
        index += 1;
        res.push((index, element_type));
    }
    res
}

pub fn show(app: &mut Application) {
    let settings = &mut app.main_settings;
    let cloned_settings = settings.clone();
    if let Some(ui) = &mut settings.ui {
        ui.window.show();
        let (main_sender, _) = app.main_channel.unwrap();

        let theme_choice = &mut ui.theme_choice;
        let shortcuts_browser = &mut ui.shortcuts_browser;

        ui.editor_text_size
            .set_value(settings.editor_text_size.into());
        ui.editor_text_size
            .emit(main_sender, Message::UpdateSettings);
        ui.reset_shortcuts
            .emit(main_sender, Message::ResetShortcuts);

        theme_choice.add(
            &Theme::get_themes()
                .iter()
                .map(|theme| format!("{}|", theme.to_string()))
                .collect::<String>(),
        );
        theme_choice.set_value(&settings.theme.to_string());
        theme_choice.emit(main_sender, Message::UpdateSettings);

        shortcuts_browser.set_column_char('\t');
        shortcuts_browser.set_column_widths(&[250]);
        shortcuts_browser.emit(main_sender, Message::UpdateSettings);
        settings.elements_shortcuts_browser_indexes = update_shortcuts_browser(
            shortcuts_browser,
            cloned_settings.clone().shortcuts_manager.shortcuts,
        );
    } else {
        settings.ui = Some(SettingsInterface::make_window());
        show(app);
    }
}

#[derive(Clone, Default, Debug)]
pub struct ShortcutsManager {
    pub shortcuts: BTreeMap<UIElementType, Shortcut>,
    pub elements: BTreeMap<UIElementType, UIElement>,
}

impl ShortcutsManager {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Register UI element
    pub fn register_ui_element(&mut self, element: &mut UIElement, element_type: UIElementType) {
        let shortcut = *self.shortcuts.get(&element_type).unwrap_or(
            ShortcutsManager::default_shortcuts()
                .get(&element_type)
                .unwrap_or(&Shortcut::None),
        );
        match element {
            UIElement::MenuItem(item) => item.set_shortcut(shortcut),
            UIElement::Button(btn) => btn.set_shortcut(shortcut),
        }
        //self.elements.insert(element_type, element.to_owned());
    }

    /// Set shortcut
    pub fn set_shortcut(&mut self, element_type: UIElementType, shortcut: Shortcut) {
        self.shortcuts.insert(element_type, shortcut);
    }

    /// Get default shortcuts
    pub fn default_shortcuts() -> BTreeMap<UIElementType, Shortcut> {
        vec![
            (UIElementType::OpenFile, Shortcut::Command | 'o'),
            (UIElementType::SaveFile, Shortcut::Command | 's'),
            (UIElementType::SaveAsFile, Shortcut::Command | 'S'),
            (UIElementType::QuitApp, Shortcut::Command | 'q'),
            (UIElementType::Copy, Shortcut::Command | 'c'),
            (UIElementType::Paste, Shortcut::Command | 'v'),
            (UIElementType::Cut, Shortcut::Command | 'x'),
            (UIElementType::Undo, Shortcut::Command | 'z'),
            (UIElementType::Redo, Shortcut::Command | 'y'),
            (UIElementType::DeleteNodeBtn, Shortcut::None | Key::Delete),
            (UIElementType::CreateNodeBtn, Shortcut::None | Key::Insert),
            (UIElementType::DelResBtn, Shortcut::None | Key::Delete),
            (UIElementType::UpNodeBtn, Shortcut::Alt | 'u'),
            (UIElementType::DownNodeBtn, Shortcut::Alt | 'd'),
        ]
        .iter()
        .map(|e| e.clone())
        .collect::<BTreeMap<_, _>>()
    }
}
