use crate::gui::state::State;
use egui::Context;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

pub fn run_background_functions(state: Arc<Mutex<State>>, gui_ctx: Arc<Mutex<Option<Context>>>) {
    loop {
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
                            let ctx = gui_ctx.lock().unwrap();
                            if let Some(ctx) = ctx.deref() {
                                ctx.request_repaint();
                            }
                        }
                    }
                }
            }
        }
        sleep(Duration::from_millis(500));
    }
}
