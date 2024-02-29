use std::env;
use crate::gui::Gui;

mod gui;
mod backend;
mod utils;

fn main() {
    let args: Vec<String> = env::args().collect();
    let gui =
        if let Some(preset_path) = args.get(1) {
            Gui::with_preset(preset_path)
        } else {
            Gui::default()
        };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 400.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Live Midi Splitter",
        options,
        Box::new(|_cc| {
            Box::new(gui)
        }),
    ).unwrap();
}

