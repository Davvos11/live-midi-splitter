use crate::backend::properties::Properties;
use crate::gui::state::State;
use egui::{Color32, ComboBox, Context, RichText, TextBuffer, Ui};
use rfd::FileDialog;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

pub fn quick_start(
    ui: &mut Ui,
    ctx: &Context,
    properties: &Arc<Mutex<Properties>>,
    loading: &Arc<Mutex<bool>>,
    state: &Arc<Mutex<State>>,
) {
    ui.heading("Quick start:");
    ui.separator();

    let mut error = state
        .lock()
        .unwrap()
        .pipewire_error
        .clone()
        .map(|e| format!("Error getting buffer size:\n{e}"));
    if let Some(pipewire) = &mut state.lock().unwrap().pipewire_status {
        let values = pipewire.get_values();

        ui.label(format!(
            "Buffer size: {}. Sample rate: {} kHz. Delay: {:.1} ms",
            values.buffer_size, values.sample_rate, values.delay
        ));

        let text = values
            .force_buffer_size
            .map(|s| s.to_string())
            .unwrap_or_default();
        ui.horizontal(|ui| {
            ui.label("Force buffer size:");
            ComboBox::from_id_source("buffer_size")
                .selected_text(text)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(values.force_buffer_size.is_none(), "None")
                        .clicked()
                    {
                        pipewire.set_buffer_size(0);
                    };
                    for i in 2..=11 {
                        let value = 2u32.pow(i);
                        if ui
                            .selectable_label(
                                values.force_buffer_size.unwrap_or_default() == value,
                                value.to_string(),
                            )
                            .clicked()
                        {
                            pipewire.set_buffer_size(value)
                        }
                    }
                });
        });
    } else if error.is_none() {
        error = Some(
            "Buffer size settings are only supported if pipewire and pw-metadata are installed"
                .into(),
        );
    }
    if let Some(error) = error {
        ui.label(RichText::new(error).color(Color32::GRAY));
    }

    ui.separator();

    ui.heading("Shortcuts:");

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
                if ui
                    .button(RichText::new(
                        egui_phosphor::regular::FOLDER_OPEN.to_string(),
                    ))
                    .clicked()
                {
                    show_dialog = Some(i);
                }
                if ui.button("X").clicked() {
                    to_remove.push(i);
                }
            });
        });

        to_remove.iter().for_each(|&i| {
            shortcuts.remove(i);
        });

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
            if let Some(location) = FileDialog::new().pick_file() {
                let mut properties = properties.lock().unwrap();
                let shortcut = properties.shortcuts.get_mut(i).unwrap();
                *shortcut = location
                    .to_str()
                    .unwrap_or("Error getting location")
                    .to_string()
            }
            *loading.lock().unwrap() = false;
        });
    }
}
