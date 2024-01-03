use eframe::Frame;
use egui::Context;

pub struct Gui {

}

impl Default for Gui {
    fn default() -> Self {
        Self {

        }
    }
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Live Midi Splitter");
        });
    }
}