use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use eframe::Frame;
use egui::Context;
use egui::panel::{Side, TopBottomSide};
use egui_dnd::dnd;

use crate::backend::Backend;
use crate::backend::preset::Preset;
use crate::backend::properties::Properties;
use crate::gui::data::RecentFiles;
use crate::gui::state::{State, TabState};
use crate::gui::tabs::input_settings::input_settings;
use crate::gui::tabs::preset::preset_tab;
use crate::gui::tabs::quick_start::quick_start;
use crate::gui::tabs::recent_files::recent_files;
use crate::gui::tabs::Tab;
use crate::gui::widgets::save_load::save_load;
use crate::gui::widgets::transpose::transpose;
use crate::utils::load;

pub mod tabs;
mod widgets;
pub mod data;
pub mod state;

pub struct Gui {
    properties: Arc<Mutex<Properties>>,
    state: Arc<Mutex<State>>,
    ctx_reference: Arc<Mutex<Option<Context>>>,

    current_tab: Arc<Mutex<Tab>>,
    tab_state: TabState,
    loading: Arc<Mutex<bool>>,

    recent_files: Arc<Mutex<RecentFiles>>,
}

impl Default for Gui {
    fn default() -> Self {
        // Start backend
        let mut backend = Backend::new();
        let properties = backend.properties();
        let state = backend.state();
        let ctx_reference = backend.gui_ctx();

        let _ = thread::spawn(move || backend.run());

        // Load recent files
        let recent_files = Arc::new(Mutex::new(RecentFiles::default()));
        // TODO maybe on a thread, but it needs to be ready for Gui::with_preset()
        if let Some(data) = RecentFiles::load() {
            *recent_files.lock().unwrap() = data;
        }

        Self {
            properties,
            state,
            ctx_reference,
            current_tab: Arc::new(Mutex::default()),
            loading: Arc::new(Mutex::new(false)),
            recent_files,
            tab_state: TabState::default(),
        }
    }
}

impl Gui {
    pub fn with_preset(path: &String) -> Self {
        let gui = Gui::default();
        let path = PathBuf::from(path);
        load(&path, Arc::clone(&gui.properties), Arc::clone(&gui.current_tab));
        gui.recent_files.lock().unwrap().add(path);

        gui
    }
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        {
            let mut ctx_reference = self.ctx_reference.lock().unwrap();
            if ctx_reference.is_none() {
                *ctx_reference = Some(ctx.clone());
            }
        }

        let mut change_preset_to = None;
        let mut delete_preset = None;
        let mut duplicate_preset = None;

        egui::TopBottomPanel::new(TopBottomSide::Top, "header").show(ctx, |ui| {
            egui::Grid::new("header-grid")
                .show(ui, |ui| {
                    ui.heading("Live MIDI splitter");
                    save_load(ui, &self.properties, &self.loading, &self.recent_files, Arc::clone(&self.current_tab));
                    let mut properties = self.properties.lock().unwrap();
                    transpose(ui, &mut properties.transpose);
                    ui.end_row();
                });
        });

        egui::SidePanel::new(Side::Left, "sidebar")
            .default_width(100.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut current_tab = self.current_tab.lock().unwrap();

                    ui.selectable_value(&mut *current_tab, Tab::RecentFiles, "Recent files");
                    ui.selectable_value(&mut *current_tab, Tab::InputSettings, "Input settings");
                    ui.selectable_value(&mut *current_tab, Tab::QuickStart, "Quick start");
                    ui.separator();
                    ui.label("Presets:");

                    let mut properties = self.properties.lock().unwrap();
                    let current_preset = properties.current_preset;
                    let presets = &mut properties.presets;
                    let drag_response = dnd(ui, "presets").show(presets.iter(), |ui, preset, handle, _| {
                        handle.ui(ui, |ui| {
                            let tab = Tab::Preset(preset.id);
                            let button = ui.selectable_label(
                                *current_tab == tab || current_preset == preset.id,
                                preset.name.clone(),
                            );
                            if button.clicked() {
                                *current_tab = tab;
                                // Besides changing the current tab, also change the preset
                                change_preset_to = Some(preset.id);
                            }
                            button.context_menu(|ui| {
                                if ui.button("Duplicate").clicked() {
                                    duplicate_preset = Some(preset.id);
                                    ui.close_menu();
                                }
                                if ui.button("Delete").clicked() {
                                    delete_preset = Some(preset.id);
                                    ui.close_menu();
                                }
                            });
                        });
                    });

                    if let Some(update) = drag_response.final_update() {
                        // Update the current preset accordingly
                        if current_preset == update.from {
                            let new_index = if update.to > update.from { update.to - 1 } else { update.to };
                            change_preset_to = Some(new_index)
                        } else if current_preset > update.from && current_preset < update.to {
                            change_preset_to = Some(current_preset - 1)
                        } else {
                            change_preset_to = Some(current_preset + 1)
                        }
                        // Change tab already (otherwise you see the previous preset at this index for one frame)
                        if let Some(id) = change_preset_to {
                            *current_tab = Tab::Preset(id);
                        }
                        // Update the vector of presets
                        drag_response.update_vec(presets);
                        // Update "internal" ids to match position in list
                        presets.iter_mut().enumerate().for_each(|(i, p)| p.id = i);
                    }

                    if ui.button("Add preset").clicked() {
                        presets.push(Preset::new_from_id(presets.len()));
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if *self.loading.lock().unwrap() {
                ui.centered_and_justified(|ui| {
                    ui.heading("Loading...");
                });
                return;
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut current_tab = self.current_tab.lock().unwrap();
                match *current_tab {
                    Tab::RecentFiles => {
                        recent_files(ui, &self.properties, &self.loading, Arc::clone(&self.recent_files), Arc::clone(&self.current_tab));
                    }
                    Tab::QuickStart => {
                        quick_start(ui, ctx, &self.properties, &self.loading, &self.state);
                    }
                    Tab::InputSettings => {
                        input_settings(ui, Arc::clone(&self.properties), Arc::clone(&self.state), &mut self.tab_state);
                    }
                    Tab::Preset(id) => {
                        // Handle preset change by backend
                        {
                            let properties = self.properties.lock().unwrap();
                            let current_preset = properties.current_preset;
                            if current_preset != id {
                                if current_preset < properties.presets.len() {
                                    *current_tab = Tab::Preset(current_preset)
                                } else {
                                    *current_tab = Tab::InputSettings
                                }
                                ctx.request_repaint();
                            }
                        }

                        preset_tab(ui, Arc::clone(&self.properties), Arc::clone(&self.state), id, &mut self.tab_state)
                    }
                }
            });
        });

        let mut properties = self.properties.lock().unwrap();
        if let Some(new_preset) = change_preset_to {
            properties.current_preset = new_preset;
        }
        if let Some(id) = delete_preset {
            properties.remove_preset(id);
        }
        if let Some(id) = duplicate_preset {
            properties.duplicate_preset(id);
        }
    }
}