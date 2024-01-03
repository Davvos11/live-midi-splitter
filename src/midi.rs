use std::error::Error;
use std::io::{stdin, stdout, Write};
use std::sync::{Arc, Mutex};
use midir::{MidiInput, MidiIO, MidiOutput, MidiOutputConnection};


pub struct Backend {
    inputs: triple_buffer::Input<Vec<String>>,
    outputs: triple_buffer::Input<Vec<String>>,
}

impl Backend {
    pub fn new(inputs: triple_buffer::Input<Vec<String>>, outputs: triple_buffer::Input<Vec<String>>) -> Self {
        Self { inputs, outputs }
    }

    pub fn run(&mut self) {
        // TODO error to frontend
        let midi_in = MidiInput::new("Live Midi Splitter in").unwrap();
        let midi_out = MidiOutput::new("Live Midi Splitter out").unwrap();
        loop {
            self.inputs.write(get_ports(&midi_in));
            self.outputs.write(get_ports(&midi_out));
        }
    }
}

fn get_ports<T: MidiIO>(midi_io: &T) -> Vec<String> {
    midi_io.ports().iter()
        .map(|p|midi_io.port_name(p).unwrap_or("Cannot get port name".to_string()))
        .collect()
}

pub fn old() {
    let outs_1: Arc<Mutex<Vec<MidiOutputConnection>>> = Arc::new(Mutex::new(Vec::new()));
    let outs_2 = Arc::clone(&outs_1);

    let midi_in = MidiInput::new("Live Midi Splitter in").unwrap();
    let in_port = select_port(&midi_in, "Input").unwrap();
    let _in_con =
        midi_in.connect(
            &in_port,
            "Input",
            move |_, message, _| {
                let outs = Arc::clone(&outs_1);
                let mut outs = outs.lock().unwrap();
                for out in outs.iter_mut() {
                    out.send(message).unwrap();
                }
            },
            ()
        ).unwrap();


    loop {
        let midi_out = MidiOutput::new("Live Midi Splitter out").unwrap();
        let out_port = select_port(&midi_out, "Output").unwrap();
        let out_con = midi_out.connect(&out_port, "Output").unwrap();

        let outs = Arc::clone(&outs_2);
        let mut outs = outs.lock().unwrap();
        outs.push(out_con);
    }
}

fn select_port<T: MidiIO>(midi_io: &T, descr: &str) -> Result<T::Port, Box<dyn Error>> {
    println!("Available {} ports:", descr);
    let midi_ports = midi_io.ports();
    for (i, p) in midi_ports.iter().enumerate() {
        println!("{}: {}", i, midi_io.port_name(p)?);
    }
    print!("Please select {} port: ", descr);
    stdout().flush()?;
    let mut input = String::new();
    stdin().read_line(&mut input)?;
    let port = midi_ports
        .get(input.trim().parse::<usize>()?)
        .ok_or("Invalid port number")?;
    println!();
    Ok(port.clone())
}