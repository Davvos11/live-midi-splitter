use eframe::emath;
use egui::{Color32, ComboBox, DragValue, RichText, Slider, TextStyle, Ui};
use egui::collapsing_header::CollapsingState;
use egui::style::{Selection, Widgets};
use egui_extras::{Column, TableBuilder};
use crate::backend::output_settings::{CcMapping, OutputSettings};
use crate::gui::state::TabState;
use crate::utils::{midi_to_note, note_to_midi};

#[derive(PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    None,
    Advanced,
    NoteFilter,
    CcMap,
}


pub fn mapping_settings(ui: &mut Ui, output_settings: &mut OutputSettings, input_id: usize, tab_state: &mut TabState) {
    let unique_id = format!("{}-{}", input_id, output_settings.port_name);
    let current_tab = tab_state.mapping_tabs.entry(unique_id.clone()).or_default();

    // ui.separator();

    let mut header = CollapsingState::load_with_default_open(
        ui.ctx(), ui.make_persistent_id(format!("advanced-{unique_id}")), false,
    );
    let is_open = header.is_open();

    if header.is_open() && *current_tab == Tab::None {
        *current_tab = Tab::Advanced;
    } else if !header.is_open() && *current_tab != Tab::None {
        header.set_open(true);
    }

    let collapse = header.show_header(ui, |ui| {
        ui.selectable_value(current_tab, Tab::Advanced, RichText::new("Advanced").text_style(TextStyle::Small));
        ui.selectable_value(current_tab, Tab::NoteFilter, RichText::new("Note filter").text_style(TextStyle::Small));
        ui.selectable_value(current_tab, Tab::CcMap, RichText::new("CC").text_style(TextStyle::Small));
    }).body(|ui| {
        match current_tab {
            Tab::None => {}
            Tab::Advanced => {
                // TODO add info button or hover, explaining the setting.
                ui.checkbox(
                    &mut output_settings.buffer_pedals,
                    RichText::new("Send pedal events after switching presets"),
                );
            }
            Tab::NoteFilter => {
                ui.checkbox(
                    &mut output_settings.key_filter_enabled,
                    RichText::new("Enable note filter"),
                );

                ui.vertical(|ui| {
                    // Hacky way to fill the slider from the current value to the end:
                    ui.visuals_mut().widgets.inactive.bg_fill = Selection::default().bg_fill;
                    ui.visuals_mut().selection.bg_fill = Widgets::default().inactive.bg_fill;
                    ui.add(
                        Slider::new(&mut output_settings.key_filter.0, 0..=128)
                            .custom_formatter(|n, _| { midi_to_note(n as u8) })
                            .custom_parser(note_to_midi)
                            .trailing_fill(true)
                    );
                });

                let moved_high = ui.add(
                    Slider::new(&mut output_settings.key_filter.1, 0..=128)
                        .custom_formatter(|n, _| { midi_to_note(n as u8) })
                        .custom_parser(note_to_midi)
                        .trailing_fill(true)
                ).dragged();

                // Make sure that it is a valid range
                if moved_high && output_settings.key_filter.1 < output_settings.key_filter.0 {
                    output_settings.key_filter.0 = output_settings.key_filter.1;
                } else if output_settings.key_filter.0 > output_settings.key_filter.1 {
                    output_settings.key_filter.1 = output_settings.key_filter.0;
                }
            }
            Tab::CcMap => {
                let cc_map = &mut output_settings.cc_map;
                let mut has_duplicates = false;
                let mut to_remove = None;

                TableBuilder::new(ui)
                    .column(Column::exact(15.0))
                    .column(Column::auto())
                    .column(Column::auto())
                    .column(Column::remainder())
                    .header(13.0, |mut header| {
                        header.col(|_| {});
                        header.col(|ui| { ui.label(RichText::new("Channel").small()); });
                        header.col(|ui| { ui.label(RichText::new("CC").small()); });
                        header.col(|ui| { ui.label(RichText::new("Target").small()); });
                    })
                    .body(|mut body| {
                        // TODO zonder clone?
                        let cc_map_c = cc_map.clone();
                        let last_index = cc_map.len() - 1;

                        cc_map.iter_mut().enumerate().for_each(|(i, (ch, cc, map))| {
                            body.row(20.0, |mut row| {
                                row.col(|ui| {
                                    #[allow(clippy::collapsible_if)] // for clarity
                                    if i != last_index {
                                        if ui.button("X").clicked() {
                                            to_remove = Some(i);
                                        }
                                    }
                                });

                                let is_duplicate = cc_map_c.iter().filter(|&(e_ch, e_cc, _)| e_ch == ch && e_cc == cc).count() > 1;
                                has_duplicates |= is_duplicate;

                                row.col(|ui| {
                                    if is_duplicate {
                                        ui.style_mut().visuals.override_text_color = Some(Color32::RED);
                                    }
                                    ui.add(
                                        filter_value_selector(ch, 0.0)
                                            .clamp_range(if i != last_index { 0..=16 } else { 0..=0 })
                                    );
                                });
                                row.col(|ui| {
                                    if is_duplicate {
                                        ui.style_mut().visuals.override_text_color = Some(Color32::RED);
                                    }
                                    ui.add(
                                        filter_value_selector(cc, -1.0)
                                            .clamp_range(if i != last_index { -1..=128 } else { -1..=-1 })
                                    );
                                });
                                row.col(|ui| {
                                    ui.horizontal(|ui| {
                                        ComboBox::from_id_source(format!("cc-target-{unique_id}-{i}"))
                                            .selected_text(map.get_description())
                                            .show_ui(ui, |ui| {
                                                CcMapping::all().clone().map(|option| {
                                                    ui.selectable_value(map, option.clone(), option.get_description_with_blanks());
                                                });
                                            });

                                        match map {
                                            CcMapping::MapToCc(cc) | CcMapping::MapToChannelCc(_, cc) => {
                                                ui.add(
                                                    DragValue::new(cc).speed(0.3).clamp_range(0..=128)
                                                );
                                            }
                                            _ => {}
                                        }

                                        if let CcMapping::MapToChannelCc(_, _) = map {
                                            ui.label("to channel");
                                        }

                                        match map {
                                            CcMapping::PassThroughToChannel(ch) | CcMapping::MapToChannelCc(ch, _) => {
                                                ui.add(
                                                    DragValue::new(ch).speed(0.3).clamp_range(1..=16)
                                                );
                                            }
                                            _ => {}
                                        }
                                    });
                                });
                            });
                        });
                    });
                if has_duplicates {
                    ui.label(RichText::new("There are duplicate rules").color(Color32::RED).text_style(TextStyle::Small));
                }
                if ui.button("Add rule").clicked() {
                    cc_map.insert(cc_map.len() - 1, (0, 0, CcMapping::default()));
                }
                if let Some(i) = to_remove {
                    cc_map.remove(i);
                }
            }
        }
    });

    if is_open { 
        ui.add_space(3.0);
        ui.separator();
        ui.add_space(3.0);
    }
    
    // If collapse button clicked
    if collapse.0.clicked() {
        if *current_tab == Tab::None {
            *current_tab = Tab::Advanced
        } else {
            *current_tab = Tab::None
        }
    }

}

fn filter_value_selector<Num: emath::Numeric>(value: &mut Num, any_value: f64) -> DragValue {
    DragValue::new(value)
        .custom_formatter(move |v, _| {
            if v == any_value { "any".to_string() } else { v.to_string() }
        })
        .custom_parser(move |s| {
            if s == "any" || s.is_empty() {
                Some(any_value)
            } else {
                s.parse().ok()
            }
        })
        .speed(0.3)
}