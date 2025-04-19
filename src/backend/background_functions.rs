use crate::backend::properties::Properties;
use crate::gui::state::State;
use crate::utils::{repaint_gui, serialise_properties};
use egui::Context;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

pub fn run_background_functions(
    state: Arc<Mutex<State>>,
    gui_ctx: Arc<Mutex<Option<Context>>>,
    properties: Arc<Mutex<Properties>>,
) {
    let mut serialised = serialise_properties(&properties.lock().unwrap());

    loop {
        // Update Pipewire Info
        {
            let mut state = state.lock().unwrap();
            if let Some(pipewire) = &mut state.pipewire_status {
                match pipewire.update() {
                    Err(err) => {
                        state.pipewire_error = Some(err.to_string());
                    }
                    Ok(updated) => {
                        state.pipewire_error = None;
                        if updated {
                            repaint_gui(&gui_ctx);
                        }
                    }
                }
            }
        }
        // Check if data has changed
        {
            let mut repaint = false;
            let serialised_new = serialise_properties(&properties.lock().unwrap());
            if properties.lock().unwrap().saved {
                serialised = serialised_new;
                properties.lock().unwrap().changed = false;
                properties.lock().unwrap().saved = false;
                repaint = true;
            } else {
                let changed = serialised != serialised_new;
                let mut properties = properties.lock().unwrap();
                let changed_old = properties.changed;
                if changed != changed_old {
                    repaint = true;
                }
                properties.changed = changed;
            }
            if repaint {
                repaint_gui(&gui_ctx);
            }
        }
        sleep(Duration::from_millis(500));
    }
}
