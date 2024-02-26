# Live Midi Splitter

Control multiple MIDI instruments with one input.

Inspired by https://sourceforge.net/projects/midi-layer/,
but easier to use with Jack/Pipewire VSTs in mind and better suited for playing live.

An example of a feature for live use is that you can switch to a different preset while holding down notes of the sustain pedal.  
If you let go of the notes or pedal, the previous instruments will still get the off messages and not hold the notes forever.

## Features / to-do

- [X] Connect MIDI inputs and outputs
- [X] Multiple presets that switch where MIDI is sent to
  - [X] Switch between presets using MIDI program change
- [X] Filter MIDI ranges, i.e. keyboard split
- [X] Send note-off and pedal off events to previous preset after switching to another
- [X] Save and load state of the program

## Usage

1. Download the latest version from [Github](https://github.com/Davvos11/live-midi-splitter/releases).
2. Extract the downloaded archive.
3. Run `live-midi-splitter-version`.
   - Optionally, you can provide the path to a preset file, for example `live-midi-splitter-version "./some preset.lmsc"` 
4. - Open some software or connect some hardware that outputs and inputs midi.
   - Select the input(s) that you want to use.
   - Create preset(s) with the output(s) you want to send to.

## Examples:

![Screenshot_20240107_015515](https://github.com/Davvos11/live-midi-splitter/assets/20478740/ef4f3367-d0cd-4d34-aa2f-c143ecbc6e36)

### Switching between presets with Program Change signals:

https://github.com/Davvos11/live-midi-splitter/assets/20478740/42569ce8-50d6-4443-bdc7-896ab15b2215

### Switching between presets while holding the sustain pedal:

https://github.com/Davvos11/live-midi-splitter/assets/20478740/45e447b1-ba34-48c8-a962-da70c9fc6c4a

