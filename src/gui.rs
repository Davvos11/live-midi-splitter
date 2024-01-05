use std::sync::{Arc, Mutex};
use std::thread;
use eframe::Frame;
use egui::Context;
use egui::panel::{Side, TopBottomSide};
use crate::gui::tabs::input_settings::input_settings;
use crate::gui::tabs::preset::preset_tab;
use crate::gui::tabs::Tab;
use crate::backend::{Backend, Properties};

mod tabs;

pub struct Gui {
    properties: Arc<Mutex<Properties>>,

    current_tab: Tab,
}

impl Default for Gui {
    fn default() -> Self {
        let mut backend = Backend::new();
        let properties = backend.properties();

        let _ = thread::spawn(move || backend.run());

        Self {
            properties,

            current_tab: Tab::default(),
        }
    }
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::TopBottomPanel::new(TopBottomSide::Top, "header").show(ctx, |ui| {
            ui.heading("Live Midi Splitter");
        });

        egui::SidePanel::new(Side::Left, "sidebar").show(ctx, |ui| {
            ui.selectable_value(&mut self.current_tab, Tab::InputSettings, "Input settings");
            ui.separator();
            ui.label("Presets:");
            ui.selectable_value(&mut self.current_tab, Tab::Preset(0), "Preset 0");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab {
                Tab::InputSettings => {
                    input_settings(ui, Arc::clone(&self.properties));
                }
                Tab::Preset(id) => {
                    preset_tab(ui, id)
                }
            }



        });
    }
}