use std::sync::{Arc, Mutex};
use egui::Ui;
use crate::backend::input_settings::InputSettings;
use crate::backend::properties::Properties;


pub fn input_settings(ui: &mut Ui, properties: Arc<Mutex<Properties>>) {
    ui.heading("Input settings");

    let mut properties = properties.lock().unwrap();
    let available_inputs = properties.available_inputs.clone();
    let mut inputs_to_remove = Vec::new();

    properties.inputs.iter_mut().enumerate().for_each(|(i, input)| {
        ui.horizontal(|ui| {
            if ui.button("X").clicked() {
                inputs_to_remove.push(i);
            }
            ui.label(format!("Input {}:", i + 1));
        });
        egui::ComboBox::from_id_source(format!("input-{i}"))
            .selected_text(&input.port_name)
            .width(200.0)
            .wrap(true)
            .show_ui(ui, |ui| {
                ui.style_mut().wrap = Some(true);
                available_inputs.iter().for_each(|input_option| {
                    ui.selectable_value(&mut input.port_name, input_option.clone(), input_option);
                });
            });

        ui.checkbox(&mut input.use_program_change, "Use Program Change to switch presets");

        ui.separator();
    });

    if ui.button("Add input").clicked() {
        properties.inputs.push(InputSettings::default());
    }

    inputs_to_remove.iter().for_each(|&i| { properties.inputs.remove(i); });
}
