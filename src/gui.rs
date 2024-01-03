use std::sync::{Arc, Mutex};
use std::thread;
use eframe::Frame;
use egui::Context;
use triple_buffer::triple_buffer;
use crate::midi::Backend;
use crate::midi::preset::Preset;

pub struct Gui {
    inputs: triple_buffer::Output<Vec<String>>,
    outputs: triple_buffer::Output<Vec<String>>,
    presets: Arc<Mutex<Vec<Preset>>>,
}

impl Default for Gui {
    fn default() -> Self {
        let (inputs_backend, inputs_frontend) = triple_buffer(&Vec::new());
        let (outputs_backend, outputs_frontend) = triple_buffer(&Vec::new());
        let presets_frontend = Arc::new(Mutex::new(vec![Preset::new("default".to_string())]));
        let presets_backend = Arc::clone(&presets_frontend);

        let mut backend = Backend::new(inputs_backend, outputs_backend, presets_backend);

        let _ = thread::spawn(move || backend.run());

        Self {
            inputs: inputs_frontend,
            outputs: outputs_frontend,
            presets: presets_frontend,
        }
    }
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Live Midi Splitter");

            for preset in self.presets.lock().unwrap().iter_mut() {
                self.inputs.read().iter().for_each(|in_port| {
                    let mut enabled = preset.inputs.contains(in_port);
                    let checkbox = ui.checkbox(&mut enabled, in_port);
                    if checkbox.changed() {
                        if enabled {
                            preset.inputs.insert(in_port.clone());
                        } else {
                            preset.inputs.remove(in_port);
                        }
                    }
                });

                ui.separator();

                self.outputs.read().iter().for_each(|in_port| {
                    let mut enabled = preset.outputs.contains(in_port);
                    let checkbox = ui.checkbox(&mut enabled, in_port);
                    if checkbox.changed() {
                        if enabled {
                            preset.outputs.insert(in_port.clone());
                        } else {
                            preset.outputs.remove(in_port);
                        }
                    }
                });
            }
        });

    }
}