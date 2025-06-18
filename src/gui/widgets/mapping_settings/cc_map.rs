use crate::backend::common_settings::{CcMap, CcMapping};
use eframe::epaint::Color32;
use egui::{Button, ComboBox, DragValue, RichText, TextStyle, Ui};
use egui_extras::{Column, TableBuilder};

use crate::gui::widgets::mapping_settings::filter_value_selector;

pub fn cc_map_settings(ui: &mut Ui, cc_map: &mut CcMap, unique_id: String) {
    let mut has_duplicates = false;
    let mut to_remove = None;

    TableBuilder::new(ui)
        .column(Column::exact(15.0))
        .column(Column::auto())
        .column(Column::auto())
        .column(Column::remainder())
        .header(13.0, |mut header| {
            header.col(|_| {});
            header.col(|ui| {
                ui.label(RichText::new("Channel").small());
            });
            header.col(|ui| {
                ui.label(RichText::new("CC").small());
            });
            header.col(|ui| {
                ui.label(RichText::new("Target").small());
            });
        })
        .body(|mut body| {
            // TODO zonder clone?
            let cc_map_c = cc_map.clone();
            let last_index = cc_map.len() - 1;

            cc_map
                .iter_mut()
                .enumerate()
                .for_each(|(i, (ch, cc, map))| {
                    body.row(20.0, |mut row| {
                        row.col(|ui| {
                            if ui.add_enabled(i != last_index, Button::new("X")).clicked() {
                                to_remove = Some(i);
                            }
                        });

                        let is_duplicate = cc_map_c
                            .iter()
                            .filter(|&(e_ch, e_cc, _)| e_ch == ch && e_cc == cc)
                            .count()
                            > 1;
                        has_duplicates |= is_duplicate;

                        row.col(|ui| {
                            if is_duplicate {
                                ui.style_mut().visuals.override_text_color = Some(Color32::RED);
                            }
                            ui.add_enabled(
                                i != last_index,
                                filter_value_selector(ch, 0.0).clamp_range(0..=16),
                            );
                        });
                        row.col(|ui| {
                            if is_duplicate {
                                ui.style_mut().visuals.override_text_color = Some(Color32::RED);
                            }
                            ui.add_enabled(
                                i != last_index,
                                filter_value_selector(cc, -1.0).clamp_range(-1..=128),
                            );
                        });
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                ComboBox::from_id_source(format!("cc-target-{unique_id}-{i}"))
                                    .selected_text(map.get_description())
                                    .show_ui(ui, |ui| {
                                        CcMapping::all().clone().map(|option| {
                                            ui.selectable_value(
                                                map,
                                                option.clone(),
                                                option.get_description_with_blanks(),
                                            );
                                        });
                                    });

                                match map {
                                    CcMapping::MapToCc(cc) | CcMapping::MapToChannelCc(_, cc) => {
                                        ui.add(DragValue::new(cc).speed(0.3).clamp_range(0..=128));
                                    }
                                    _ => {}
                                }

                                if let CcMapping::MapToChannelCc(_, _) = map {
                                    ui.label("to channel");
                                }

                                match map {
                                    CcMapping::PassThroughToChannel(ch)
                                    | CcMapping::MapToChannelCc(ch, _) => {
                                        ui.add(DragValue::new(ch).speed(0.3).clamp_range(1..=16));
                                    }
                                    _ => {}
                                }
                            });
                        });
                    });
                });
        });
    if has_duplicates {
        ui.label(
            RichText::new("There are duplicate rules")
                .color(Color32::RED)
                .text_style(TextStyle::Small),
        );
    }
    if ui.button("Add rule").clicked() {
        cc_map.insert(cc_map.len() - 1, (0, 0, CcMapping::default()));
    }
    if let Some(i) = to_remove {
        cc_map.remove(i);
    }
}
