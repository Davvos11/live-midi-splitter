use std::sync::{Arc, Mutex};
use std::thread;
use eframe::Frame;
use egui::Context;
use egui::panel::{Side, TopBottomSide};
use crate::gui::tabs::input_settings::input_settings;
use crate::gui::tabs::preset::preset_tab;
use crate::gui::tabs::Tab;
use crate::backend::Backend;
use crate::backend::preset::Preset;
use crate::backend::properties::Properties;
use crate::gui::widgets::save_load::save_load;

mod tabs;
mod widgets;

pub struct Gui {
    properties: Arc<Mutex<Properties>>,
    ctx_reference: Arc<Mutex<Option<Context>>>,

    current_tab: Tab,
    loading: Arc<Mutex<bool>>,
}

impl Default for Gui {
    fn default() -> Self {
        let mut backend = Backend::new();
        let properties = backend.properties();
        let ctx_reference = backend.gui_ctx();

        let _ = thread::spawn(move || backend.run());

        Self {
            properties,
            ctx_reference,
            current_tab: Tab::default(),
            loading: Arc::new(Mutex::new(false)),
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
                save_load(ui, &self.properties, &self.loading);
            });
        });

        egui::SidePanel::new(Side::Left, "sidebar")
            .default_width(100.0)
            .show(ctx, |ui| {
                ui.selectable_value(&mut self.current_tab, Tab::InputSettings, "Input settings");
                ui.separator();
                ui.label("Presets:");

                self.properties.lock().unwrap()
                    .presets.iter().enumerate()
                    .for_each(|(i, preset)| {
                        if ui.selectable_value(&mut self.current_tab, Tab::Preset(i), preset.name.clone())
                            .clicked() {
                            // Besides changing the current tab, also change the preset
                            change_preset_to = Some(i);
                        }
                    });
                if ui.button("Add preset").clicked() {
                    self.properties.lock().unwrap()
                        .presets.push(Preset::default());
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