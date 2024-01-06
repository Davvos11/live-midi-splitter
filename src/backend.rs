use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use midir::MidiIO;
use crate::backend::device::{ConnectError, Input, new_input, new_output, Output};
use crate::backend::preset::Preset;

pub mod preset;
mod device;

pub struct Properties {
    pub available_inputs: Vec<String>,
    pub available_outputs: Vec<String>,
    pub inputs: Vec<String>,
    pub presets: Vec<Preset>,
    pub current_preset: usize,
}

impl Default for Properties {
    fn default() -> Self {
        Self {
            available_inputs: Vec::new(),
            available_outputs: Vec::new(),
            inputs: vec![String::new()],
            presets: vec![Preset::default()],
            current_preset: 0,
        }
    }
}


pub struct Backend {
    properties: Arc<Mutex<Properties>>,

    input_listeners: Vec<Input>,
    output_handlers: Arc<Mutex<HashMap<String, Output>>>,
}

impl Backend {
    pub fn new() -> Self {
        Self {
            properties: Arc::new(Mutex::new(Properties::default())),

            input_listeners: Vec::new(),
            output_handlers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn run(&mut self) {
        // TODO error to frontend (new_input uses unwrap)
        let midi_in = new_input();
        let midi_out = new_output();
        loop {
            let mut properties = self.properties.lock().unwrap();
            // Send available ports to frontend
            properties.available_inputs = get_ports(&midi_in);
            properties.available_outputs = get_ports(&midi_out);

            // New input factory:
            let new_listener = |name: String, input_id: usize| {
                let properties = Arc::clone(&self.properties);
                let output_handlers = Arc::clone(&self.output_handlers);
                Input::new(
                    name,
                    move |_, data| {
                        let properties = properties.lock().unwrap();
                        if let Some(preset) = properties.presets.get(properties.current_preset) {
                            if let Some(mapping) = preset.mapping.get(&input_id) {
                                mapping.iter().for_each(|output_name| {
                                    let mut output_handlers = output_handlers.lock().unwrap();
                                    if !output_handlers.contains_key(output_name) {
                                        // Try to connect
                                        let new_handler = Output::new(output_name);
                                        match new_handler {
                                            Ok(handler) => {
                                                output_handlers.insert(output_name.clone(), handler);
                                            }
                                            Err(_) => { return; }
                                        }
                                    }
                                    // We can unwrap because we checked or inserted the item above
                                    output_handlers.get_mut(output_name).unwrap()
                                        .connection.send(data).unwrap_or_else(|_| println!("Failed to send to {}", output_name));
                                });
                            }
                        }
                    },
                )
            };

            // Update input listeners (and count them)
            let listener_count = properties.inputs.iter()
                .filter(|s| !s.is_empty())
                .enumerate()
                .map(|(i, input_name)| {
                    if let Some(input) = self.input_listeners.get_mut(i) {
                        if input.port_name != *input_name {
                            // Input setting has changed, change connection
                            *input = new_listener(input_name.clone(), i);
                        }
                    } else {
                        // New input, add new connection
                        self.input_listeners.push(new_listener(input_name.clone(), i));
                    }
                }).count();
            // Remove deleted input listeners
            self.input_listeners.drain(listener_count..);

            drop(properties);

            thread::sleep(Duration::from_millis(100));
        }
    }


    pub fn properties(&self) -> Arc<Mutex<Properties>> {
        Arc::clone(&self.properties)
    }
}

fn get_ports<T: MidiIO>(midi_io: &T) -> Vec<String> {
    midi_io.ports().iter()
        .map(|p| midi_io.port_name(p).unwrap_or("Cannot get port name".to_string()))
        .filter(|p| !p.starts_with("Live Midi Splitter"))
        .collect()
}

