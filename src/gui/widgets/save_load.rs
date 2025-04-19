use crate::backend::properties::Properties;
use crate::gui::data::RecentFiles;
use crate::gui::keybinds::{keybind_button, Keybinds};
use crate::gui::state::State;
use crate::gui::tabs::Tab;
use crate::utils::{load_dialog, save, save_dialog};
use egui::Ui;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

pub fn save_load(
    ui: &mut Ui,
    properties: &Arc<Mutex<Properties>>,
    loading: &Arc<Mutex<bool>>,
    recent_files: &Arc<Mutex<RecentFiles>>,
    current_tab: &Arc<Mutex<Tab>>,
    state: &Arc<Mutex<State>>,
    keybinds: &Keybinds,
) {
    ui.horizontal(|ui| {
        if keybind_button(ui, "Open", &keybinds.load, true).clicked() {
            gui_load(properties, loading, recent_files, current_tab, state)
        }
        
        let changed = properties.lock().unwrap().changed;

        if let Some(filename) = state.lock().unwrap().file_path().clone() {
            if keybind_button(ui, "Save", &keybinds.save, changed).clicked() {
                gui_save(filename, properties, loading)
            }
        }

        if keybind_button(ui, "Save as", &keybinds.save_as, changed).clicked() {
            gui_save_as(properties, loading, recent_files, state)
        }
    });
}

pub fn gui_load(
    properties: &Arc<Mutex<Properties>>,
    loading: &Arc<Mutex<bool>>,
    recent_files: &Arc<Mutex<RecentFiles>>,
    current_tab: &Arc<Mutex<Tab>>,
    state: &Arc<Mutex<State>>,
) {
    let loading = Arc::clone(loading);
    let properties = Arc::clone(properties);
    let current_tab = Arc::clone(current_tab);
    let recent_files = Arc::clone(recent_files);
    let state = Arc::clone(state);
    let _ = thread::spawn(move || {
        *loading.lock().unwrap() = true;
        if let Some(file) = load_dialog(properties, current_tab) {
            let mut recent_files = recent_files.lock().unwrap();
            state.lock().unwrap().set_file_path(file.clone());
            recent_files.add(file);
        }
        *loading.lock().unwrap() = false;
    });
}

pub fn gui_save(
    filename: PathBuf,
    properties: &Arc<Mutex<Properties>>,
    loading: &Arc<Mutex<bool>>,
) {
    let loading = Arc::clone(loading);
    let properties = Arc::clone(properties);
    let _ = thread::spawn(move || {
        *loading.lock().unwrap() = true;
        save(&filename, properties);
        *loading.lock().unwrap() = false;
    });
}

pub fn gui_save_as(
    properties: &Arc<Mutex<Properties>>,
    loading: &Arc<Mutex<bool>>,
    recent_files: &Arc<Mutex<RecentFiles>>,
    state: &Arc<Mutex<State>>,
) {
    let loading = Arc::clone(loading);
    let properties = Arc::clone(properties);
    let recent_files = Arc::clone(recent_files);
    let state = Arc::clone(state);
    let _ = thread::spawn(move || {
        *loading.lock().unwrap() = true;
        if let Some(file) = save_dialog(properties) {
            let mut recent_files = recent_files.lock().unwrap();
            state.lock().unwrap().set_file_path(file.clone());
            recent_files.add(file);
        }
        *loading.lock().unwrap() = false;
    });
}
