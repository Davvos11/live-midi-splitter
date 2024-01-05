use std::sync::{Arc, Mutex};
use egui::Ui;
use crate::midi::Properties;


pub fn input_settings(ui: &mut Ui, properties: Arc<Mutex<Properties>>) {
    ui.heading("Input settings");

    let mut properties = properties.lock().unwrap();
    let available_inputs = properties.available_inputs.clone();
    let mut inputs_to_remove = Vec::new();

    if properties.inputs.is_empty() {
        properties.inputs.push(String::new());
    }

    properties.inputs.iter_mut().enumerate().for_each(|(i, input)| {
        ui.horizontal(|ui| {
            if ui.button("X").clicked() {
                inputs_to_remove.push(i);
            }
            ui.label(format!("Input {}:", i + 1));
        });
        egui::ComboBox::from_id_source(format!("input-{i}"))
            .selected_text(input.clone())
            .width(200.0)
            .wrap(true)
            .show_ui(ui, |ui| {
                ui.style_mut().wrap = Some(true);
                available_inputs.iter().for_each(|input_option| {
                    ui.selectable_value(input, input_option.clone(), input_option);
                });
            });
        ui.separator();
    });

    if ui.button("Add input").clicked() {
        properties.inputs.push(String::new());
    }

    inputs_to_remove.iter().for_each(|&i| { properties.inputs.remove(i); });
}
