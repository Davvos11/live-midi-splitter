use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use midir::MidiIO;
use crate::backend::device::{Input, new_input, new_output};
use crate::backend::preset::Preset;

pub mod preset;
mod device;

pub struct Properties {
    pub available_inputs: Vec<String>,
    pub available_outputs: Vec<String>,
    pub presets: Vec<Preset>,
    pub inputs: Vec<String>,
}

impl Default for Properties {
    fn default() -> Self {
        Self {
            available_inputs: Vec::new(),
            available_outputs: Vec::new(),
            presets: vec![Preset::default()],
            inputs: vec![String::new()]
        }
    }
}


pub struct Backend {
    properties: Arc<Mutex<Properties>>,

    input_listeners: Vec<Input>,
}

impl Backend {
    pub fn new() -> Self {
        Self {
            properties: Arc::new(Mutex::new(Properties::default())),

            input_listeners: Vec::new(),
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
            let new_listener = |name: String| {
                Input::new(
                    name,
                    |_, data| { dbg!(data); },
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
                            *input = new_listener(input_name.clone());
                        }
                    } else {
                        // New input, add new connection
                        self.input_listeners.push(new_listener(input_name.clone()));
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

