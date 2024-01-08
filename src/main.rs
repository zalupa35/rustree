#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod event_loop;
mod formats;
mod history_manager;
mod settings;
mod types;
mod ui;
mod utils;

use app::Application;

fn main() {
    let mut app = Application::new();
    app.run();
}
