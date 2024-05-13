use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use egui::Context;
use midir::MidiIO;
use midly::live::LiveEvent;
use midly::num::{u4, u7};

use crate::backend::device::{Input, new_input, new_output, Output};
use crate::backend::midi_handler::{create_new_listener, EventBufferItem};
use crate::backend::properties::Properties;
use crate::gui::state::State;

pub mod preset;
pub mod properties;
pub mod input_settings;
pub mod midi_handler;
mod device;
pub mod output_settings;
pub mod common_settings;

pub struct Backend {
    properties: Arc<Mutex<Properties>>,
    state: Arc<Mutex<State>>,
    gui_ctx: Arc<Mutex<Option<Context>>>,

    input_listeners: Vec<Input>,
    output_handlers: Arc<Mutex<HashMap<String, Output>>>,
    event_buffer: Arc<Mutex<HashMap<LiveEvent<'static>, HashSet<EventBufferItem>>>>,
    held_pedals: Arc<Mutex<HashMap<(u4, u7), u7>>>, // (channel, controller): value
}

impl Backend {
    pub fn new() -> Self {
        Self {
            properties: Arc::new(Mutex::new(Properties::default())),
            state: Arc::new(Mutex::new(Default::default())),
            gui_ctx: Arc::new(Mutex::new(None)),

            input_listeners: Vec::new(),
            output_handlers: Arc::new(Mutex::new(HashMap::new())),
            event_buffer: Arc::new(Mutex::new(HashMap::new())),
            held_pedals: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn run(&mut self) {
        // TODO error to frontend (new_input uses unwrap)
        let midi_in = new_input();
        let midi_out = new_output();
        loop {
            {
                let mut properties = self.properties.lock().unwrap();
                let mut state = self.state.lock().unwrap();

                // Send available ports to frontend
                state.available_inputs = get_ports(&midi_in);
                state.available_outputs = get_ports(&midi_out);

                // New input factory:
                let new_listener = |name, input_id| {
                    create_new_listener(
                        name,
                        input_id,
                        Arc::clone(&self.properties),
                        Arc::clone(&self.state),
                        Arc::clone(&self.gui_ctx),
                        Arc::clone(&self.output_handlers),
                        Arc::clone(&self.event_buffer),
                        Arc::clone(&self.held_pedals),
                    )
                };

                // Update input listeners
                properties.inputs.iter()
                    .filter(|s| !s.port_name.is_empty())
                    .enumerate()
                    .for_each(|(i, new_input)| {
                        if let Some(input) = self.input_listeners.get_mut(i) {
                            if input.port_name != *new_input.port_name {
                                // Input setting has changed, change connection
                                if let Ok(new_input) = new_listener(new_input.port_name.clone(), i) {
                                    *input = new_input;
                                }
                            }
                        } else {
                            // New input, add new connection
                            if let Ok(new_input) = new_listener(new_input.port_name.clone(), i) {
                                self.input_listeners.push(new_input);
                            }
                        }
                    });
                // Remove disconnected and removed input listeners
                self.input_listeners.retain(|input| {
                    // Remove input listeners that do not exist anymore
                    state.available_inputs.contains(&input.port_name) &&
                        // Remove input listeners that are not selected by the user anymore
                        properties.inputs.iter().filter(|i| i.port_name == input.port_name).count() > 0
                });
            }
            thread::sleep(Duration::from_millis(100));
        }
    }


    pub fn properties(&self) -> Arc<Mutex<Properties>> {
        Arc::clone(&self.properties)
    }


    pub fn gui_ctx(&self) -> Arc<Mutex<Option<Context>>> {
        Arc::clone(&self.gui_ctx)
    }
    
    pub fn state(&self) -> Arc<Mutex<State>> {
        Arc::clone(&self.state)
    }
}

fn get_ports<T: MidiIO>(midi_io: &T) -> Vec<String> {
    midi_io.ports().iter()
        .map(|p| midi_io.port_name(p).unwrap_or("Cannot get port name".to_string()))
        .filter(|p| !p.starts_with("testLive Midi Splitter"))
        .collect()
}

