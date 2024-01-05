use egui::Ui;


pub fn preset_tab(ui: &mut Ui, id: usize) {
    ui.heading(format!("Preset {}", id));
}
