use egui::{Button, Key, KeyboardShortcut, ModifierNames, Modifiers, Response, Ui, WidgetText};
use egui_keybind::{Bind, Shortcut};

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

pub fn keybind_button(
    ui: &mut Ui,
    text: impl Into<WidgetText>,
    shortcut: &Shortcut,
    enabled: bool,
) -> Response {
    let btn = Button::new(text);
    let btn = ui.add_enabled(enabled, btn);
    btn.on_hover_text(shortcut.format(&ModifierNames::NAMES, false))
}
