# Live Midi Splitter

Control multiple MIDI instruments with one input.

The goal is something similar to https://sourceforge.net/projects/midi-layer/,
but easier to use with Jack/Pipewire VSTs in mind.

## Features / to-do

- [X] Connect MIDI inputs and outputs
- [X] Multiple presets that switch where MIDI is sent to
  - [X] Switch between presets using MIDI program change
- [ ] Filter MIDI ranges, i.e. keyboard split
- [ ] Send note-off events to previous preset after switching to another
- [X] Save and load state of the program

## Usage

1. Download the latest version from [Github](https://github.com/Davvos11/live-midi-splitter/releases).
2. Extract the downloaded archive.
3. Run `live-midi-splitter`.
4. - Open some software or connect some hardware that outputs and inputs midi.
   - Select the input(s) that you want to use.
   - Create preset(s) with the output(s) you want to send to.

## Example:

![Screenshot_20240107_015515](https://github.com/Davvos11/live-midi-splitter/assets/20478740/ef4f3367-d0cd-4d34-aa2f-c143ecbc6e36)

https://github.com/Davvos11/live-midi-splitter/assets/20478740/42569ce8-50d6-4443-bdc7-896ab15b2215
