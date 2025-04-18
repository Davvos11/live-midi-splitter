use egui::{Key, KeyboardShortcut, Modifiers};
use egui_keybind::Shortcut;

pub struct Keybinds {
    pub load: Shortcut,
    pub save: Shortcut,
    pub save_as: Shortcut,
}

impl Default for Keybinds {
    fn default() -> Self {
        Self {
            load: keyboard_shortcut(Modifiers::CTRL, Key::O),
            save: keyboard_shortcut(Modifiers::CTRL, Key::S),
            save_as: keyboard_shortcut(Modifiers::CTRL | Modifiers::SHIFT, Key::S),
        }
    }
}

fn keyboard_shortcut(modifiers: Modifiers, logical_key: Key) -> Shortcut {
    Shortcut::new(Some(KeyboardShortcut::new(modifiers, logical_key)), None)
}
