use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::thread;
use eframe::Frame;
use egui::Context;
use egui::panel::{Side, TopBottomSide};
use egui_dnd::dnd;
use crate::gui::tabs::input_settings::input_settings;
use crate::gui::tabs::preset::preset_tab;
use crate::gui::tabs::Tab;
use crate::backend::Backend;
use crate::backend::preset::Preset;
use crate::backend::properties::Properties;
use crate::gui::data::RecentFiles;
use crate::gui::tabs::recent_files::recent_files;
use crate::gui::widgets::save_load::save_load;

mod tabs;
mod widgets;
pub mod data;

pub struct Gui {
    properties: Arc<Mutex<Properties>>,
    ctx_reference: Arc<Mutex<Option<Context>>>,

    current_tab: Tab,
    loading: Arc<Mutex<bool>>,

    recent_files: Arc<Mutex<RecentFiles>>
}

impl Default for Gui {
    fn default() -> Self {
        // Start backend
        let mut backend = Backend::new();
        let properties = backend.properties();
        let ctx_reference = backend.gui_ctx();

        let _ = thread::spawn(move || backend.run());

        // Load recent files
        let recent_files = Arc::new(Mutex::new(RecentFiles::default()));
        let recent_files_thread = Arc::clone(&recent_files);
        let _ = thread::spawn(move || {
            if let Some(data) = RecentFiles::load() {
                *recent_files_thread.lock().unwrap() = data;
            }
        });

        Self {
            properties,
            ctx_reference,
            current_tab: Tab::default(),
            loading: Arc::new(Mutex::new(false)),
            recent_files,
        }
    }
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        let mut ctx_reference = self.ctx_reference.lock().unwrap();
        if ctx_reference.is_none() {
            *ctx_reference = Some(ctx.clone());
        }

        let mut change_preset_to = None;

        egui::TopBottomPanel::new(TopBottomSide::Top, "header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Live Midi Splitter");
                save_load(ui, &self.properties, &self.loading, &self.recent_files);
            });
        });

        egui::SidePanel::new(Side::Left, "sidebar")
            .default_width(100.0)
            .show(ctx, |ui| {
                ui.selectable_value(&mut self.current_tab, Tab::RecentFiles, "Recent files");
                ui.selectable_value(&mut self.current_tab, Tab::InputSettings, "Input settings");
                ui.separator();
                ui.label("Presets:");

                let mut properties = self.properties.lock().unwrap();
                let current_preset = properties.current_preset;
                let presets = &mut properties.presets;
                let drag_response = dnd(ui, "presets").show(presets.iter(), |ui, preset, handle, _| {
                    handle.ui(ui, |ui| {
                        if ui.selectable_value(&mut self.current_tab, Tab::Preset(preset.id), preset.name.clone())
                                    .changed() {
                            // Besides changing the current tab, also change the preset
                            change_preset_to = Some(preset.id);
                        }
                    });
                });

                if let Some(update) = drag_response.final_update() {
                    // Update the current preset accordingly
                    if current_preset == update.from {
                        let new_index = if update.to > update.from { update.to -1 } else { update.to };
                        change_preset_to = Some(new_index)
                    } else if current_preset > update.from && current_preset < update.to {
                        change_preset_to = Some(current_preset - 1)
                    } else {
                        change_preset_to = Some(current_preset + 1)
                    }
                    // Change tab already (otherwise you see the previous preset at this index for one frame)
                    if let Some(id) = change_preset_to {
                        self.current_tab = Tab::Preset(id);
                    }
                    // Update the vector of presets
                    drag_response.update_vec(presets);
                    // Update "internal" ids to match position in list
                    presets.iter_mut().enumerate().for_each(|(i, p)|p.id = i);
                }

                if ui.button("Add preset").clicked() {
                    presets.push(Preset::new_from_id(presets.len()));
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if *self.loading.lock().unwrap() {
                ui.centered_and_justified(|ui| {
                    ui.heading("Loading...");
                });
                return;
            }

            match self.current_tab {
                Tab::RecentFiles => {
                    recent_files(ui, &self.properties, &self.loading, Arc::clone(&self.recent_files));
                }
                Tab::InputSettings => {
                    input_settings(ui, Arc::clone(&self.properties));
                }
                Tab::Preset(id) => {
                    // Handle preset change by backend
                    {
                        let properties = self.properties.lock().unwrap();
                        let current_preset = properties.current_preset;
                        if current_preset != id {
                            if current_preset < properties.presets.len() {
                                self.current_tab = Tab::Preset(current_preset)
                            } else {
                                self.current_tab = Tab::InputSettings
                            }
                            ctx.request_repaint();
                        }
                    }

                    preset_tab(ui, Arc::clone(&self.properties), id)
                }
            }
        });

        if let Some(new_preset) = change_preset_to {
            self.properties.lock().unwrap().current_preset = new_preset;
        }
    }
}