# Live Midi Splitter

Control multiple MIDI instruments with one input.

Inspired by https://sourceforge.net/projects/midi-layer/,
but easier to use with Jack/Pipewire VSTs in mind and better suited for playing live.

https://github.com/Davvos11/live-midi-splitter/assets/20478740/6d5f8208-849c-4c1c-88c6-42b0ba117915

## Features / to-do

- [X] Connect MIDI inputs and outputs
- [X] Multiple presets that switch where MIDI is sent to
  - [X] Switch between presets using MIDI program change
  - [ ] Allow switching using MIDI pads and send feedback
  - [ ] Preset groups/variants
- [X] Filter MIDI ranges, i.e. keyboard split
- [X] Send note-off and pedal off events to previous preset after switching to another
- [X] Save and load state of the program
  - [ ] Auto save
- [x] Filter and map MIDI CC
- [x] Filter and map MIDI channels
- [x] Velocity curves

## Usage

1. Download the latest version from [Github](https://github.com/Davvos11/live-midi-splitter/releases).
2. Extract the downloaded archive.
3. Run `live-midi-splitter-version`.
   - Optionally, you can provide the path to a preset file, for example `live-midi-splitter-version "./some preset.lmsc"` 
4. - Open some software or connect some hardware that outputs and inputs midi.
   - Select the input(s) that you want to use.
   - Create preset(s) with the output(s) you want to send to.

## Old examples:
**A lot of these screenshots are of older versions of the software**

![Screenshot_20240107_015515](https://github.com/Davvos11/live-midi-splitter/assets/20478740/ef4f3367-d0cd-4d34-aa2f-c143ecbc6e36)

### Switching between presets with Program Change signals:

https://github.com/Davvos11/live-midi-splitter/assets/20478740/42569ce8-50d6-4443-bdc7-896ab15b2215

### Keyboard split / note filter

https://github.com/Davvos11/live-midi-splitter/assets/20478740/d3fc13cf-b49b-4844-b59a-027949fc0d40

### Midi CC map and filter

https://github.com/Davvos11/live-midi-splitter/assets/20478740/780a1523-806d-4ed3-be7e-b6cad260b0a6

### Switching between presets while holding the sustain pedal:

https://github.com/Davvos11/live-midi-splitter/assets/20478740/45e447b1-ba34-48c8-a962-da70c9fc6c4a
