#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MidiMessage {
    /// Channel 0–15
    pub channel: u8,
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

impl MidiMessage {
    pub fn from_bytes(data: [u8; 3], time: u32) -> Self {
        let [status, data1, data2] = data;
        let channel = status & 0x0F;
        let kind = match status & 0xF0 {
            0x80 => MidiMessageKind::NoteOff {
                note: data1,
                velocity: data2,
            },
            0x90 => MidiMessageKind::NoteOn {
                note: data1,
                velocity: data2,
            },
            0xB0 => MidiMessageKind::ControlChange {
                controller: data1,
                value: data2,
            },
            0xE0 => MidiMessageKind::PitchBend {
                // two 7-bit values, LSB first
                value: ((data2 as i16) << 7 | data1 as i16) - 8192,
            },
            _ => MidiMessageKind::Unknown {
                status,
                data1,
                data2,
            },
        };

        Self {
            channel,
            kind,
            time,
        }
    }
}
