use std::sync::{Arc, Mutex};
use egui::Ui;
use crate::backend::Properties;


pub fn preset_tab(ui: &mut Ui, properties: Arc<Mutex<Properties>>, id: usize) {
    let mut properties = properties.lock().unwrap();
    let inputs = properties.inputs.clone();
    let available_outputs = properties.available_outputs.clone();

    if let Some(preset) = properties.presets.get_mut(id) {
        egui::TextEdit::singleline(&mut preset.name).show(ui);
        ui.separator();

        inputs.iter().enumerate().for_each(|(input_id, input)| {
            ui.label(input);
            let mapping = preset.mapping.entry(input_id).or_default();
            let mut maps_to_remove = Vec::new();

            mapping.iter_mut().enumerate().for_each(|(map_id, output)| {
                ui.horizontal(|ui| {
                    if ui.button("X").clicked() {
                        maps_to_remove.push(map_id);
                    }
                    egui::ComboBox::from_id_source(format!("mapping-{input_id}-{map_id}"))
                        .selected_text(output.clone())
                        .width(0.5) // FIXME width not working
                        .wrap(true)
                        .show_ui(ui, |ui| {
                            ui.style_mut().wrap = Some(true);
                            available_outputs.iter().for_each(|output_option| {
                                ui.selectable_value(output, output_option.clone(), output_option);
                            });
                        });
                });
            });

            if ui.button("Add output").clicked() {
                mapping.push(String::new());
            }
            maps_to_remove.iter().for_each(|&i| { mapping.remove(i); });
        });

    } else {
        ui.heading("Failed to load preset");
    }
}
