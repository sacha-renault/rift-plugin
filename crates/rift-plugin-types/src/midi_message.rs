use clack_plugin::events::Event;

mod midi_1_0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MidiMessage {
    /// Channel 0–15
    pub channel: u8,
    pub port_index: u16,
    pub kind: MidiMessageKind,
    pub time: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MidiMessageKind {
    NoteOn { note: u8, velocity: u8 },
    NoteOff { note: u8, velocity: u8 },
    ControlChange { controller: u8, value: u8 },
    PitchBend { value: i16 },
    Unknown { status: u8, data1: u8, data2: u8 },
}
