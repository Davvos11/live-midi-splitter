use std::sync::{Arc, Mutex};
use egui::{RichText, Ui};
use crate::backend::properties::Properties;

pub fn transpose(ui: &mut Ui, properties: Arc<Mutex<Properties>>) {
    let mut properties = properties.lock().unwrap();

    egui::Grid::new("transpose-wrapper")
        .spacing([2.0, 0.0])
        .show(ui, |ui| {
            ui.label("Transpose:");
            ui.add(egui::DragValue::new(&mut properties.transpose)
                .clamp_range(-12..=12),
            );
            egui::Grid::new("transpose-buttons")
                .spacing([0.0, -10.0])
                .show(ui, |ui| {
                    let height = ui.available_height() / 3.0;
                    if ui.small_button(RichText::new("⏶").size(height)).clicked() {
                        properties.transpose += 1;
                    }
                    ui.end_row();
                    if ui.small_button(RichText::new("⏷").size(height)).clicked() {
                        properties.transpose -= 1;
                    }
                    ui.end_row();
                });
            ui.end_row();
        });
}