use crate::gui::Gui;

mod gui;
mod backend;

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Live Midi Splitter",
        options,
        Box::new(|_cc| {
            Box::<Gui>::default()
        }),
    ).unwrap();
}

