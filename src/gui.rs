use std::thread;
use eframe::Frame;
use egui::Context;
use triple_buffer::triple_buffer;
use crate::midi::Backend;

pub struct Gui {
    inputs: triple_buffer::Output<Vec<String>>,
    outputs: triple_buffer::Output<Vec<String>>,
}

impl Default for Gui {
    fn default() -> Self {
        let (inputs_backend, inputs_frontend) = triple_buffer(&Vec::new());
        let (outputs_backend, outputs_frontend) = triple_buffer(&Vec::new());

        let mut backend = Backend::new(inputs_backend, outputs_backend);

        let _ = thread::spawn(move || backend.run());

        Self {
            inputs: inputs_frontend,
            outputs: outputs_frontend,
        }
    }
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Live Midi Splitter");

            self.inputs.read().iter().for_each(|in_port| {
                ui.label(format!("Input: {in_port}"));
            });
            self.outputs.read().iter().for_each(|in_port| {
                ui.label(format!("Output: {in_port}"));
            });
        });
    }
}