use crate::backend::properties::Properties;
use egui::{Context, RichText, TextBuffer, Ui};
use rfd::FileDialog;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use egui_modal::Modal;

pub fn quick_start(ui: &mut Ui,
                   ctx: &Context,
                   properties: &Arc<Mutex<Properties>>,
                   loading: &Arc<Mutex<bool>>,
) {
    ui.heading("Quick start:");

    ui.separator();

    let mut show_dialog = None;
    {
        let shortcuts = &mut properties.lock().unwrap().shortcuts;
        let mut to_remove = Vec::new();

        shortcuts.iter_mut().enumerate().for_each(|(i, shortcut)| {
            ui.horizontal(|ui| {
                if ui.button("Launch").clicked() {
                    let location = shellexpand::tilde(&shortcut);
                    let location = PathBuf::from(location.as_str());
                    if let Err(e) = open::that_detached(location) {
                        // TODO modal or smth
                        egui::Window::new("Error launching shortcut")
                            .open(&mut true)
                            .show(ctx, |ui| {
                                ui.code(e.to_string());
                            });
                    }
                }
                egui::TextEdit::singleline(shortcut)
                    .desired_width(ui.available_width() - 60.0)
                    .show(ui);
                if ui.button(RichText::new(egui_phosphor::regular::FOLDER_OPEN.to_string())).clicked() {
                    show_dialog = Some(i);
                }
                if ui.button("X").clicked() {
                    to_remove.push(i);
                }
            });
        });

        to_remove.iter().for_each(|&i| { shortcuts.remove(i); });

        if ui.button("Add shortcut").clicked() {
            shortcuts.push(String::default());
        }
    }
    
    // "open" button:
    if let Some(i) = show_dialog {
        let loading = Arc::clone(loading);
        let properties = Arc::clone(properties);
        let _ = thread::spawn(move || {
            *loading.lock().unwrap() = true;
            if let Some(location) = FileDialog::new()
                .pick_file() {
                let mut properties = properties.lock().unwrap();
                let shortcut = properties.shortcuts.get_mut(i).unwrap();
                *shortcut = location.to_str().unwrap_or("Error getting location").to_string()
            }
            *loading.lock().unwrap() = false;
        });
    }
}