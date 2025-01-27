use std::env;

use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::format::FmtSpan;

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

    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    eframe::run_native(
        "Live Midi Splitter",
        options,
        Box::new(|cc| {
            let mut fonts = egui::FontDefinitions::default();
            egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

            cc.egui_ctx.set_fonts(fonts);
            Box::new(gui)
        }),
    ).unwrap();
}

