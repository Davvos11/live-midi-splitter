use egui::collapsing_header::CollapsingState;
use egui::{RichText, TextStyle, Ui};
use crate::backend::common_settings::CommonSettings;
use crate::backend::input_settings::InputSettings;
use crate::gui::state::TabState;
use crate::gui::widgets::mapping_settings::cc_map::cc_map_settings;
use crate::gui::widgets::mapping_settings::note_filter::note_filter_settings;
use crate::gui::widgets::mapping_settings::velocity_map::velocity_map_settings;

#[derive(PartialEq, Eq, Default)]
pub enum InputTab {
    #[default]
    None,
    Advanced,
    NoteFilter,
    CcMap,
    VelocityMap,
}

pub fn input_mapping_settings(ui: &mut Ui, input_settings: &mut InputSettings, input_id: usize, tab_state: &mut TabState) {
    let unique_id = format!("{}", input_id);
    let current_tab = tab_state.input_tabs.entry(input_id).or_default();

    // ui.separator();

    let mut header = CollapsingState::load_with_default_open(
        ui.ctx(), ui.make_persistent_id(format!("advanced-{unique_id}")), false,
    );
    let is_open = header.is_open();

    if header.is_open() && *current_tab == InputTab::None {
        *current_tab = InputTab::Advanced;
    } else if !header.is_open() && *current_tab != InputTab::None {
        header.set_open(true);
    }

    let collapse = header.show_header(ui, |ui| {
        // ui.selectable_value(current_tab, InputTab::Advanced, RichText::new("Advanced").text_style(TextStyle::Small));
        ui.selectable_value(current_tab, InputTab::NoteFilter, RichText::new("Notes").text_style(TextStyle::Small));
        ui.selectable_value(current_tab, InputTab::VelocityMap, RichText::new("Velocity").text_style(TextStyle::Small));
        ui.selectable_value(current_tab, InputTab::CcMap, RichText::new("CC").text_style(TextStyle::Small));
    }).body(|ui| {
        match current_tab {
            InputTab::None => {}
            InputTab::Advanced => {
                ui.label("Hello :)");
            }
            InputTab::NoteFilter => {
                note_filter_settings(ui, input_settings, unique_id);
            }
            InputTab::CcMap => {
                cc_map_settings(ui, input_settings.cc_map_mut(), unique_id);
            }
            InputTab::VelocityMap => {
                velocity_map_settings(ui, input_settings, unique_id);
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
        if *current_tab == InputTab::None {
            *current_tab = InputTab::Advanced
        } else {
            *current_tab = InputTab::None
        }
    }
}