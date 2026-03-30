pub const A5_MIDI: f32 = 81.;
pub const A5_FREQUENCY: f32 = 440.;

pub fn midi_to_frequency(midi_note: u8) -> f32 {
    let offset = midi_note as f32 - A5_MIDI;
    A5_FREQUENCY * 2.0_f32.powf(offset / 12.)
}
