use eframe::epaint::Color32;
use egui::{Button, ComboBox, DragValue, RichText, Slider, TextStyle, Ui};
use egui::style::{Selection, Widgets};
use egui_extras::{Column, TableBuilder};

use crate::backend::common_settings::CommonSettings;
use crate::backend::output_settings::ChannelMapping;
use crate::gui::widgets::mapping_settings::filter_value_selector;
use crate::utils::{midi_to_note, note_to_midi};

pub fn note_filter_settings(ui: &mut Ui, settings: &mut impl CommonSettings, unique_id: String) {
    ui.checkbox(
        settings.key_filter_enabled_mut(),
        RichText::new("Enable note filter"),
    );

    ui.vertical(|ui| {
        // Hacky way to fill the slider from the current value to the end:
        ui.visuals_mut().widgets.inactive.bg_fill = Selection::default().bg_fill;
        ui.visuals_mut().selection.bg_fill = Widgets::default().inactive.bg_fill;
        ui.add(
            Slider::new(&mut settings.key_filter_mut().0, 0..=128)
                .custom_formatter(|n, _| { midi_to_note(n as u8) })
                .custom_parser(note_to_midi)
                .trailing_fill(true)
        );
    });

    let moved_high = ui.add(
        Slider::new(&mut settings.key_filter_mut().1, 0..=128)
            .custom_formatter(|n, _| { midi_to_note(n as u8) })
            .custom_parser(note_to_midi)
            .trailing_fill(true)
    ).dragged();

    // Make sure that it is a valid range
    if moved_high && settings.key_filter_mut().1 < settings.key_filter_mut().0 {
        settings.key_filter_mut().0 = settings.key_filter_mut().1;
    } else if settings.key_filter_mut().0 > settings.key_filter_mut().1 {
        settings.key_filter_mut().1 = settings.key_filter_mut().0;
    }

    ui.separator();

    let channel_map = settings.channel_map_mut();
    let mut has_duplicates = false;
    let mut to_remove = None;

    TableBuilder::new(ui)
        .column(Column::exact(15.0))
        .column(Column::auto())
        .column(Column::remainder())
        .header(13.0, |mut header| {
            header.col(|_| {});
            header.col(|ui| { ui.label(RichText::new("Channel").small()); });
            header.col(|ui| { ui.label(RichText::new("Target").small()); });
        })
        .body(|mut body| {
            // TODO zonder clone?
            let cc_map_c = channel_map.clone();
            let last_index = channel_map.len() - 1;

            channel_map.iter_mut().enumerate().for_each(|(i, (ch_in, ch_out))| {
                body.row(20.0, |mut row| {
                    row.col(|ui| {
                        if ui.add_enabled(i != last_index, Button::new("X")).clicked() {
                            to_remove = Some(i);
                        }
                    });

                    let is_duplicate = cc_map_c.iter().filter(|&(e_ch, _)| e_ch == ch_in).count() > 1;
                    has_duplicates |= is_duplicate;

                    row.col(|ui| {
                        if is_duplicate {
                            ui.style_mut().visuals.override_text_color = Some(Color32::RED);
                        }
                        ui.add_enabled(
                            i != last_index,
                            filter_value_selector(ch_in, 0.0).clamp_range(0..=16),
                        );
                    });
                    row.col(|ui| {
                        ui.horizontal(|ui| {
                            ComboBox::from_id_source(format!("ch-target-{unique_id}-{i}"))
                                .selected_text(ch_out.get_description())
                                .show_ui(ui, |ui| {
                                    ChannelMapping::all().clone().map(|option| {
                                        ui.selectable_value(ch_out, option.clone(), option.get_description_with_blanks());
                                    });
                                });

                            if let ChannelMapping::Channel(cc) = ch_out {
                                ui.add(
                                    DragValue::new(cc).speed(0.3).clamp_range(0..=16)
                                );
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
        channel_map.insert(channel_map.len() - 1, (1, ChannelMapping::default()));
    }
    if let Some(i) = to_remove {
        channel_map.remove(i);
    }
}