use std::sync::{Arc, Mutex};
use std::thread;
use egui::{RichText, TextStyle, Ui};
use crate::backend::properties::Properties;
use crate::gui::data::RecentFiles;
use crate::utils::load;

pub fn recent_files(ui: &mut Ui,
                    properties: &Arc<Mutex<Properties>>,
                    loading: &Arc<Mutex<bool>>,
                    recent_files: &RecentFiles,
) {
    ui.heading("Recent:");

    ui.separator();

    recent_files.files.iter().for_each(|file| {
        if let Some(file_path) = file.to_str() {
            let file_name = file.file_name().and_then(|s| s.to_str())
                .unwrap_or("[unknown file]");
            if ui.link(file_name).clicked() {
                let loading = Arc::clone(loading);
                let properties = Arc::clone(properties);
                let location = file.clone();
                let _ = thread::spawn(move || {
                    *loading.lock().unwrap() = true;
                    let _ = load(&location, properties);
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