use std::sync::{Arc, Mutex};

use egui::{Frame, Margin, Rgba, RichText, Rounding, Ui};

use crate::backend::output_settings::OutputSettings;
use crate::backend::properties::Properties;
use crate::gui::state::{State, TabState};
use crate::gui::widgets::mapping_settings::mapping_settings;

pub fn preset_tab(ui: &mut Ui, properties: Arc<Mutex<Properties>>, state: Arc<Mutex<State>>, id: usize, tab_state: &mut TabState) {
    let mut properties = properties.lock().unwrap();
    let state = state.lock().unwrap();
    
    let inputs = properties.inputs.clone();
    let available_outputs = state.available_outputs.clone();

    let mut remove_preset = false;

    if let Some(preset) = properties.presets.get_mut(id) {
        ui.horizontal(|ui| {
            egui::TextEdit::singleline(&mut preset.name)
                .desired_width(ui.available_width() - 60.0)
                .show(ui);
            remove_preset = ui.button("Remove").clicked();
        });

        inputs.iter().enumerate().for_each(|(input_id, input)| {
            Frame::default()
                .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                .rounding(Rounding::same(5.0))
                .inner_margin(5.0)
                .outer_margin(Margin { left: 0.0, right: 0.0, top: 5.0, bottom: 0.0 })
                .show(ui, |ui| {
                    ui.label(&input.port_name);
                    let mapping = preset.mapping.entry(input_id).or_default();
                    let mut maps_to_remove = Vec::new();

                    mapping.iter_mut().enumerate().for_each(|(map_id, output)| {
                        ui.horizontal(|ui| {
                            if ui.button("X").clicked() {
                                maps_to_remove.push(map_id);
                            }
                            // Colour red if the selected output is not available (anymore)
                            let text = if available_outputs.iter().any(|p| p.readable == output.port_name) {
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
                                        ui.selectable_value(&mut output.port_name, output_option.readable.clone(), output_option.readable.clone());
                                    });
                                });
                        });

                        mapping_settings(ui, output, input_id, tab_state);
                    });

                    if ui.button("Add output").clicked() {
                        mapping.push(OutputSettings::default());
                    }

                    maps_to_remove.iter().for_each(|&i| { mapping.remove(i); });
                });
        });
    } else {
        ui.heading("Failed to load preset");
    }

    if remove_preset {
        properties.remove_preset(id);
    }
}
