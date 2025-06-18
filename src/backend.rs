use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::backend::device::{new_input, new_output, Input, Output};
use crate::backend::midi_handler::{EventBufferItem, Listener, QueueHandler, QueueItems};
use crate::backend::properties::Properties;
use crate::gui::state::State;
use egui::Context;
use midir::MidiIO;
use midly::live::LiveEvent;
use midly::num::{u4, u7};
use once_cell::sync::Lazy;
use regex::Regex;

pub mod background_functions;
pub mod common_settings;
mod device;
pub mod input_settings;
pub mod midi_handler;
pub mod output_settings;
pub mod pipewire_utils;
pub mod preset;
pub mod properties;

pub struct Backend {
    properties: Arc<Mutex<Properties>>,
    state: Arc<Mutex<State>>,
    gui_ctx: Arc<Mutex<Option<Context>>>,

    input_listeners: Vec<Input>,
    output_handlers: Arc<Mutex<HashMap<String, Output>>>,
    event_buffer: Arc<Mutex<HashMap<LiveEvent<'static>, HashSet<EventBufferItem>>>>,
    held_pedals: Arc<Mutex<HashMap<(u4, u7), u7>>>, // (channel, controller): value
    queue: Arc<Mutex<BinaryHeap<Reverse<u64>>>>,
}

impl Backend {
    pub fn new() -> Self {
        Self {
            properties: Arc::new(Mutex::new(Properties::default())),
            state: Arc::new(Mutex::new(State::new())),
            gui_ctx: Arc::new(Mutex::new(None)),

            input_listeners: Vec::new(),
            output_handlers: Arc::new(Mutex::new(HashMap::new())),
            event_buffer: Arc::new(Mutex::new(HashMap::new())),
            held_pedals: Arc::new(Mutex::new(HashMap::new())),
            queue: Arc::new(Mutex::new(BinaryHeap::new())),
        }
    }

    pub fn run(&mut self) {
        // TODO error to frontend (new_input uses unwrap)
        let midi_in = new_input();
        let midi_out = new_output();

        let (event_sender, event_receiver) = mpsc::channel::<(u64, QueueItems)>();

        let mut queue_handler =
            QueueHandler::new(Arc::clone(&self.output_handlers), Arc::clone(&self.queue));
        let _ = thread::spawn(move || queue_handler.run(event_receiver));

        loop {
            {
                let properties = self.properties.lock().unwrap();
                let mut state = self.state.lock().unwrap();

                // Send available ports to frontend
                state.available_inputs = get_ports(&midi_in);
                state.available_outputs = get_ports(&midi_out);

                // New input factory:
                let new_listener = |name, input_id| {
                    Listener {
                        name,
                        input_id,
                        properties: Arc::clone(&self.properties),
                        state: Arc::clone(&self.state),
                        gui_ctx: Arc::clone(&self.gui_ctx),
                        output_handlers: Arc::clone(&self.output_handlers),
                        event_buffer: Arc::clone(&self.event_buffer),
                        held_pedals: Arc::clone(&self.held_pedals),
                        queue: Arc::clone(&self.queue),
                        event_sender: event_sender.clone(),
                    }
                    .create()
                };

                // Update input listeners
                properties
                    .inputs
                    .iter()
                    .filter(|s| !s.port_name.is_empty())
                    .enumerate()
                    .for_each(|(i, new_input)| {
                        let port = state
                            .available_inputs
                            .iter()
                            .find(|p| p.readable == new_input.port_name);

                        if let Some(input) = self.input_listeners.get_mut(i) {
                            if input.port_name.readable != new_input.port_name {
                                // Input setting has changed, change connection
                                if let Some(Ok(new_input)) =
                                    port.map(|p| new_listener(p.clone(), i))
                                {
                                    *input = new_input;
                                }
                            }
                        } else {
                            // New input, add new connection
                            if let Some(Ok(new_input)) = port.map(|p| new_listener(p.clone(), i)) {
                                self.input_listeners.push(new_input);
                            }
                        }
                    });
                // Remove disconnected and removed input listeners
                self.input_listeners.retain(|input| {
                    // Remove input listeners that do not exist anymore
                    state.available_inputs.contains(&input.port_name) &&
                        // Remove input listeners that are not selected by the user anymore
                        properties.inputs.iter().any(|i| i.port_name == input.port_name.readable)
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

fn get_ports<T: MidiIO>(midi_io: &T) -> Vec<MidiPort> {
    midi_io
        .ports()
        .iter()
        .map(|p| {
            midi_io
                .port_name(p)
                .unwrap_or("Cannot get port name".to_string())
        })
        .map(parse_port)
        .collect()
}

#[derive(Clone, Debug, PartialEq)]
pub struct MidiPort {
    pub readable: String,
    pub internal: String,
}

impl MidiPort {
    fn new(readable: String, internal: String) -> Self {
        Self { readable, internal }
    }

    fn new_simple(internal: String) -> Self {
        Self {
            readable: internal.clone(),
            internal,
        }
    }
}

/// Helper to get more usable device names (i.e. for Midi-Bridge)
fn parse_port(port_name: String) -> MidiPort {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\((playback|capture)_\d+\) (.+)$").unwrap());
    if port_name.starts_with("Midi-Bridge:") {
        if let Some(groups) = RE.captures(&port_name) {
            if let Some(name) = groups.get(2) {
                return MidiPort::new(name.as_str().to_string(), port_name);
            }
        }
    }
    MidiPort::new_simple(port_name)
}
