use std::sync::{Arc, Mutex};
use std::thread;
use egui::Ui;

use crate::backend::properties::Properties;
use crate::gui::data::RecentFiles;
use crate::utils::{load_dialog, save_dialog};

pub fn save_load(ui: &mut Ui,
                 properties: &Arc<Mutex<Properties>>,
                 loading: &Arc<Mutex<bool>>,
                 recent_files: &Arc<Mutex<RecentFiles>>
) {
    ui.horizontal(|ui| {
        if ui.button("Open").clicked() {
            let loading = Arc::clone(loading);
            let properties = Arc::clone(properties);
            let recent_files = Arc::clone(recent_files);
            let _ = thread::spawn(move || {
                *loading.lock().unwrap() = true;
                if let Some(file) = load_dialog(properties) {
                    let mut recent_files = recent_files.lock().unwrap();
                    recent_files.add(file);
                    recent_files.save();
                }
                *loading.lock().unwrap() = false;
            });
        }

        if ui.button("Save").clicked() {
            let loading = Arc::clone(loading);
            let properties = Arc::clone(properties);
            let recent_files = Arc::clone(recent_files);
            let _ = thread::spawn(move || {
                *loading.lock().unwrap() = true;
                if let Some(file) = save_dialog(properties) {
                    let mut recent_files = recent_files.lock().unwrap();
                    recent_files.add(file);
                    recent_files.save();
                }
                *loading.lock().unwrap() = false;
            });
        }
    });
}

