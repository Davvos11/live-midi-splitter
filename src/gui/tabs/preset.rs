use std::sync::{Arc, Mutex};
use egui::{Rgba, RichText, Ui};
use crate::backend::output_settings::OutputSettings;
use crate::backend::properties::Properties;
use crate::gui::state::TabState;
use crate::gui::widgets::mapping_settings::mapping_settings;


pub fn preset_tab(ui: &mut Ui, properties: Arc<Mutex<Properties>>, id: usize, tab_state: &mut TabState) {
    let mut properties = properties.lock().unwrap();
    let inputs = properties.inputs.clone();
    let available_outputs = properties.available_outputs.clone();

    let mut remove_preset = false;

    if let Some(preset) = properties.presets.get_mut(id) {
        ui.horizontal(|ui| {
            egui::TextEdit::singleline(&mut preset.name)
                .desired_width(ui.available_width() - 60.0)
                .show(ui);
            remove_preset = ui.button("Remove").clicked();
        });
        ui.separator();

        inputs.iter().enumerate().for_each(|(input_id, input)| {
            ui.label(&input.port_name);
            let mapping = preset.mapping.entry(input_id).or_default();
            let mut maps_to_remove = Vec::new();

            mapping.iter_mut().enumerate().for_each(|(map_id, output)| {
                ui.horizontal(|ui| {
                    if ui.button("X").clicked() {
                        maps_to_remove.push(map_id);
                    }
                    // Colour red if the selected output is not available (anymore)
                    let text = if available_outputs.contains(&output.port_name) {
                        RichText::new(&output.port_name)
                    } else {
                        RichText::new(&output.port_name).color(Rgba::from_rgb(1.0, 0.0, 0.0))
                    };
                    egui::ComboBox::from_id_source(format!("mapping-{input_id}-{map_id}"))
                        .selected_text(text)
                        .width(0.5) // FIXME width not working
                        .wrap(true)
                        .show_ui(ui, |ui| {
                            ui.style_mut().wrap = Some(true);
                            available_outputs.iter().for_each(|output_option| {
                                ui.selectable_value(&mut output.port_name, output_option.clone(), output_option);
                            });
                        });
                });

                mapping_settings(ui, output, input_id, tab_state);
            });

            if ui.button("Add output").clicked() {
                mapping.push(OutputSettings::default());
            }

            ui.separator();

            maps_to_remove.iter().for_each(|&i| { mapping.remove(i); });
        });
    } else {
        ui.heading("Failed to load preset");
    }

    if remove_preset {
        properties.presets.remove(id);
        // Update "internal" ids to match position in list
        properties.presets.iter_mut().enumerate().for_each(|(i, p)| p.id = i);
        properties.current_preset = if id > 0 { id - 1 } else { 0 };
    }
}
