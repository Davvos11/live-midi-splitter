use eframe::emath;
use egui::{DragValue, RichText, TextStyle, Ui};
use egui::collapsing_header::CollapsingState;
use crate::backend::common_settings::CommonSettings;

use crate::backend::output_settings::OutputSettings;
use crate::gui::state::TabState;
use crate::gui::widgets::mapping_settings::cc_map::cc_map_settings;
use crate::gui::widgets::mapping_settings::note_filter::note_filter_settings;
use crate::gui::widgets::mapping_settings::velocity_map::velocity_map_settings;

pub mod note_filter;
pub mod cc_map;
pub mod velocity_map;

#[derive(PartialEq, Eq, Default)]
pub enum OutputTab {
    #[default]
    None,
    Advanced,
    NoteFilter,
    CcMap,
    Velocity,
}


pub fn mapping_settings(ui: &mut Ui, output_settings: &mut OutputSettings, input_id: usize, tab_state: &mut TabState) {
    let unique_id = format!("{}-{}", input_id, output_settings.port_name);
    let current_tab = tab_state.mapping_tabs.entry(unique_id.clone()).or_default();

    // ui.separator();

    let mut header = CollapsingState::load_with_default_open(
        ui.ctx(), ui.make_persistent_id(format!("advanced-{unique_id}")), false,
    );
    let is_open = header.is_open();

    if header.is_open() && *current_tab == OutputTab::None {
        *current_tab = OutputTab::Advanced;
    } else if !header.is_open() && *current_tab != OutputTab::None {
        header.set_open(true);
    }

    let collapse = header.show_header(ui, |ui| {
        ui.selectable_value(current_tab, OutputTab::Advanced, RichText::new("Advanced").text_style(TextStyle::Small));
        ui.selectable_value(current_tab, OutputTab::NoteFilter, RichText::new("Notes").text_style(TextStyle::Small));
        ui.selectable_value(current_tab, OutputTab::Velocity, RichText::new("Velocity").text_style(TextStyle::Small));
        ui.selectable_value(current_tab, OutputTab::CcMap, RichText::new("CC").text_style(TextStyle::Small));
    }).body(|ui| {
        match current_tab {
            OutputTab::None => {}
            OutputTab::Advanced => {
                // TODO add info button or hover, explaining the setting.
                ui.checkbox(
                    &mut output_settings.buffer_pedals,
                    RichText::new("Send pedal events after switching presets"),
                );
            }
            OutputTab::NoteFilter => {
                note_filter_settings(ui, output_settings, unique_id);
            }
            OutputTab::Velocity => {
                velocity_map_settings(ui, output_settings, unique_id);
            }
            OutputTab::CcMap => {
                cc_map_settings(ui, output_settings.cc_map_mut(), unique_id);
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
        if *current_tab == OutputTab::None {
            *current_tab = OutputTab::Advanced
        } else {
            *current_tab = OutputTab::None
        }
    }
}

pub fn filter_value_selector<Num: emath::Numeric>(value: &mut Num, any_value: f64) -> DragValue {
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