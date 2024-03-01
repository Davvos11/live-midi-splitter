use std::sync::{Arc, Mutex};
use std::thread;
use egui::{RichText, TextStyle, Ui};
use crate::backend::properties::Properties;
use crate::gui::data::RecentFiles;
use crate::gui::tabs::Tab;
use crate::utils::load;

pub fn recent_files(ui: &mut Ui,
                    properties: &Arc<Mutex<Properties>>,
                    loading: &Arc<Mutex<bool>>,
                    recent_files: Arc<Mutex<RecentFiles>>,
                    current_tab: Arc<Mutex<Tab>>,
) {
    ui.heading("Recent:");

    ui.separator();

    recent_files.lock().unwrap().files().iter().rev().for_each(|file| {
        if let Some(file_path) = file.to_str() {
            let file_name = file.file_name().and_then(|s| s.to_str())
                .unwrap_or("[unknown file]");
            if ui.link(file_name).clicked() {
                let loading = Arc::clone(loading);
                let properties = Arc::clone(properties);
                let location = file.clone();
                let recent_files = Arc::clone(&recent_files);
                let current_tab = Arc::clone(&current_tab);
                let _ = thread::spawn(move || {
                    *loading.lock().unwrap() = true;
                    let mut recent_files = recent_files.lock().unwrap();
                    if load(&location, properties, current_tab) {
                        recent_files.add(location);
                    } else {
                        recent_files.remove(&location);
                    }
                    *loading.lock().unwrap() = false;
                });
            }
            ui.label(
                RichText::new(file_path).text_style(TextStyle::Small)
            );
            ui.separator();
        }
    });
}