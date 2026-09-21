pub const A4_MIDI: f32 = 69.;
pub const A4_FREQUENCY: f32 = 440.;

pub fn midi_to_frequency(midi_note: u8) -> f32 {
    let offset = midi_note as f32 - A4_MIDI;
    A4_FREQUENCY * 2.0_f32.powf(offset / 12.)
}
