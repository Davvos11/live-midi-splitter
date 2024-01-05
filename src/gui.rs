use std::sync::{Arc, Mutex};
use std::thread;
use eframe::Frame;
use egui::Context;
use egui::panel::{Side, TopBottomSide};
use triple_buffer::triple_buffer;
use crate::gui::tabs::input_settings::input_settings;
use crate::gui::tabs::preset::preset_tab;
use crate::gui::tabs::Tab;
use crate::midi::Backend;
use crate::midi::preset::Preset;

mod tabs;

pub struct Gui {
    available_inputs: triple_buffer::Output<Vec<String>>,
    available_outputs: triple_buffer::Output<Vec<String>>,
    inputs: Arc<Mutex<Vec<String>>>,
    presets: Arc<Mutex<Vec<Preset>>>,

    current_tab: Tab,
}

impl Default for Gui {
    fn default() -> Self {
        let (av_inputs_backend, av_inputs_frontend) = triple_buffer(&Vec::new());
        let (av_outputs_backend, av_outputs_frontend) = triple_buffer(&Vec::new());
        let presets_frontend = Arc::new(Mutex::new(vec![Preset::new("default".to_string())]));
        let presets_backend = Arc::clone(&presets_frontend);
        let inputs_frontend = Arc::new(Mutex::new(Vec::new()));
        let inputs_backend = Arc::clone(&presets_frontend);

        let mut backend = Backend::new(av_inputs_backend, av_outputs_backend, presets_backend);

        let _ = thread::spawn(move || backend.run());

        Self {
            available_inputs: av_inputs_frontend,
            available_outputs: av_outputs_frontend,
            inputs: inputs_frontend,
            presets: presets_frontend,

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
                    input_settings(ui);
                }
                Tab::Preset(id) => {
                    preset_tab(ui, id)
                }
            }



        });
    }
}