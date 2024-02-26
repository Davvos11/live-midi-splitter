use egui::{RichText, Slider, TextStyle, Ui};
use egui::collapsing_header::CollapsingState;
use crate::backend::output_settings::OutputSettings;
use crate::gui::state::TabState;
use crate::utils::{midi_to_note, note_to_midi};

#[derive(PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    None,
    Advanced,
    NoteFilter,
}


pub fn mapping_settings(ui: &mut Ui, output_settings: &mut OutputSettings, input_id: usize, tab_state: &mut TabState) {
    let unique_id = format!("{}-{}", input_id, output_settings.port_name);
    let current_tab = tab_state.mapping_tabs.entry(unique_id.clone()).or_default();

    // ui.separator();

    let mut header = CollapsingState::load_with_default_open(
        ui.ctx(), ui.make_persistent_id(format!("advanced-{unique_id}")), false
    );

    if header.is_open() && *current_tab == Tab::None {
        *current_tab = Tab::Advanced;
    } else if !header.is_open() && *current_tab != Tab::None {
        header.set_open(true);
    }

    let collapse = header.show_header(ui, |ui| {
        ui.selectable_value(current_tab, Tab::Advanced, RichText::new("Advanced").text_style(TextStyle::Small));
        ui.selectable_value(current_tab, Tab::NoteFilter, RichText::new("Note filter").text_style(TextStyle::Small));
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
            }
        }
    });

    // If collapse button clicked
    if collapse.0.clicked() {
        if *current_tab == Tab::None {
            *current_tab = Tab::Advanced
        } else {
            *current_tab = Tab::None
        }
    }

    ui.separator();
}