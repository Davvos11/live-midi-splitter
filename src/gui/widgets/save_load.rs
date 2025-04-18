use egui::Ui;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::backend::properties::Properties;
use crate::gui::data::RecentFiles;
use crate::gui::state::State;
use crate::gui::tabs::Tab;
use crate::utils::{load_dialog, save, save_dialog};

pub fn save_load(
    ui: &mut Ui,
    properties: &Arc<Mutex<Properties>>,
    loading: &Arc<Mutex<bool>>,
    recent_files: &Arc<Mutex<RecentFiles>>,
    current_tab: Arc<Mutex<Tab>>,
    state: &Arc<Mutex<State>>,
) {
    ui.horizontal(|ui| {
        if ui.button("Open").clicked() {
            let loading = Arc::clone(loading);
            let properties = Arc::clone(properties);
            let recent_files = Arc::clone(recent_files);
            let state = Arc::clone(state);
            let _ = thread::spawn(move || {
                *loading.lock().unwrap() = true;
                if let Some(file) = load_dialog(properties, current_tab) {
                    let mut recent_files = recent_files.lock().unwrap();
                    state.lock().unwrap().file_path = Some(file.clone());
                    recent_files.add(file);
                }
                *loading.lock().unwrap() = false;
            });
        }

        if let Some(filename) = state.lock().unwrap().file_path.clone() {
            if ui.button("Save").clicked() {
                let loading = Arc::clone(loading);
                let properties = Arc::clone(properties);
                let _ = thread::spawn(move || {
                    *loading.lock().unwrap() = true;
                    save(&filename, properties);
                    *loading.lock().unwrap() = false;
                });
            }
        }

        if ui.button("Save as").clicked() {
            let loading = Arc::clone(loading);
            let properties = Arc::clone(properties);
            let recent_files = Arc::clone(recent_files);
            let state = Arc::clone(state);
            let _ = thread::spawn(move || {
                *loading.lock().unwrap() = true;
                if let Some(file) = save_dialog(properties) {
                    let mut recent_files = recent_files.lock().unwrap();
                    state.lock().unwrap().file_path = Some(file.clone());
                    recent_files.add(file);
                }
                *loading.lock().unwrap() = false;
            });
        }
    });
}
