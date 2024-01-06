# Live Midi Splitter

Control multiple MIDI instruments with one input.

Currently only a proof of concept.

The goal is something similar to https://sourceforge.net/projects/midi-layer/,
but easier to use with Jack/Pipewire VSTs in mind.

## Features / to-do

- [X] Connect MIDI inputs and outputs
- [X] Multiple presets that switch where MIDI is sent to
  - [X] Switch between presets using MIDI program change
- [ ] Filter MIDI ranges, i.e. keyboard split
- [ ] Send note-off events to previous preset after switching to another
- [ ] Save and load state of the program

## Usage

To run use `cargo run` or for an optimised version, use `cargo run --release`.

Open some programs that use MIDI inputs and connect a MIDI input.  
The MIDI of the checked inputs will be sent to all the checked outputs.

## Example:

![Screenshot_20240106_015132](https://github.com/Davvos11/live-midi-splitter/assets/20478740/2a72af14-d8e6-4d30-99bf-a06229dd1155)

https://github.com/Davvos11/live-midi-splitter/assets/20478740/38dbfb77-559a-4478-bea5-f300a9db7b78
