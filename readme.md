# Live Midi Splitter

Control multiple MIDI instruments with one input.

Currently only a proof of concept.

The goal is something similar to https://sourceforge.net/projects/midi-layer/,
but easier to use with Jack/Pipewire VSTs in mind.

## Usage

To run use `cargo run` or for an optimised version, use `cargo run --release`.

Open some programs that use MIDI inputs and connect a MIDI input.  
The MIDI of the checked inputs will be sent to all the checked outputs.

![Screenshot_20240103_233309](https://github.com/Davvos11/live-midi-splitter/assets/20478740/9ce02821-8afa-468c-b444-1aa0d901ff26)

## Features / to-do

- [X] Connect MIDI inputs and outputs
- [ ] Multiple presets that switch where MIDI is sent to
  - [ ] Switch between presets using MIDI program change
- [ ] Filter MIDI ranges, i.e. keyboard split
- [ ] Send note-off events to previous preset after switching to another
- [ ] Save and load state of the program
