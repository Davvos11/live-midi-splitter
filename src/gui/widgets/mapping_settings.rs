use std::sync::{Arc, Mutex};
use egui::{CollapsingHeader, RichText, Slider, TextStyle, Ui};
use crate::backend::output_settings::OutputSettings;
use crate::utils::{midi_to_note, note_to_midi};

pub fn mapping_settings(ui: &mut Ui, output_settings: &mut OutputSettings) {
    // TODO add info button or hover, explaining the setting.
    ui.checkbox(
        &mut output_settings.buffer_pedals,
        RichText::new("Send pedal events after switching presets").text_style(TextStyle::Small),
    );
    CollapsingHeader::new(RichText::new("Key filter").text_style(TextStyle::Small))
        .id_source(format!("key-filter-{}", output_settings.port_name))
        .show(ui, |ui| {
            ui.checkbox(
                &mut output_settings.key_filter_enabled,
                RichText::new("Enable key filter").text_style(TextStyle::Small),
            );
            ui.add(
                Slider::new(&mut output_settings.key_filter.0, 0..=128)
                    .custom_formatter(|n, _| { midi_to_note(n as u8) })
                    .custom_parser(note_to_midi)
            );
            ui.add(
                Slider::new(&mut output_settings.key_filter.1, 0..=128)
                    .custom_formatter(|n, _| { midi_to_note(n as u8) })
                    .custom_parser(note_to_midi)
            );
        });
}