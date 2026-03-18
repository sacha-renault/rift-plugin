use clack_plugin::events::{Event, event_types::MidiEvent};

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

impl From<MidiEvent> for MidiMessage {
    fn from(value: MidiEvent) -> Self {
        let time = value.time();
        let port_index = value.port_index();
        let [status, data1, data2] = value.data();
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
                value: ((data2 as i16) << 7 | data1 as i16) - 8192,
            },
            _ => MidiMessageKind::Unknown {
                status: status & 0xF0,
                data1,
                data2,
            },
        };

        Self {
            channel,
            port_index,
            kind,
            time,
        }
    }
}

impl Into<MidiEvent> for MidiMessage {
    fn into(self) -> MidiEvent {
        let channel = self.channel;

        let data = match self.kind {
            MidiMessageKind::Unknown {
                data1,
                data2,
                status,
            } => [channel | status, data1, data2],
            MidiMessageKind::NoteOff { note, velocity } => [channel | 0x80, note, velocity],
            MidiMessageKind::NoteOn { note, velocity } => [channel | 0x90, note, velocity],
            MidiMessageKind::ControlChange { controller, value } => {
                [channel | 0xB0, controller, value]
            }
            MidiMessageKind::PitchBend { value } => {
                let shifted = value + 8192;
                let data1 = (shifted & 0x7F) as u8;
                let data2 = ((shifted >> 7) & 0x7F) as u8;
                [channel | 0xE0, data1, data2]
            }
        };

        MidiEvent::new(self.time, self.port_index, data)
    }
}

#[cfg(test)]
mod tests {
    use std::u8;

    use super::*;

    #[test]
    fn tes_round_tript_note_on() {
        let start = MidiMessage {
            port_index: 1,
            channel: 4,
            time: 16,
            kind: MidiMessageKind::NoteOn {
                note: 60,
                velocity: 100,
            },
        };
        let mid: MidiEvent = start.into();
        let finish = MidiMessage::from(mid);

        assert_eq!(start, finish);
    }

    #[test]
    fn test_round_trip_note_off() {
        let start = MidiMessage {
            port_index: 0,
            channel: 0,
            time: 0,
            kind: MidiMessageKind::NoteOff {
                note: 60,
                velocity: 100,
            },
        };
        let mid: MidiEvent = start.into();
        let finish = MidiMessage::from(mid);

        assert_eq!(start, finish);
    }

    #[test]
    fn test_round_trip_controler_change() {
        let start = MidiMessage {
            port_index: 16,
            channel: 2,
            time: 3,
            kind: MidiMessageKind::ControlChange {
                controller: 1,
                value: 50,
            },
        };
        let mid: MidiEvent = start.into();
        let finish = MidiMessage::from(mid);

        assert_eq!(start, finish);
    }

    #[test]
    fn test_round_trip_pitch_bend() {
        let pitch_range = 8191;
        for value in pitch_range..=pitch_range {
            let start = MidiMessage {
                port_index: 16,
                channel: 2,
                time: 3,
                kind: MidiMessageKind::PitchBend { value },
            };
            let mid: MidiEvent = start.into();
            let finish = MidiMessage::from(mid);
            assert_eq!(start, finish);
        }
    }

    #[test]
    fn test_round_trip_unknown() {
        let start = MidiMessage {
            port_index: 16,
            channel: 2,
            time: 3,
            kind: MidiMessageKind::Unknown {
                status: 0xA0,
                data1: u8::MAX / 4,
                data2: u8::MIN,
            },
        };
        let mid: MidiEvent = start.into();
        let finish = MidiMessage::from(mid);
        assert_eq!(start, finish);
    }

    #[test]
    fn test_round_trip_all_channels() {
        for channel in 0..0x0F {
            let start = MidiMessage {
                port_index: 16,
                channel,
                time: 3,
                kind: MidiMessageKind::Unknown {
                    status: 0xA0,
                    data1: u8::MAX / 4,
                    data2: u8::MIN,
                },
            };
            let mid: MidiEvent = start.into();
            let finish = MidiMessage::from(mid);
            assert_eq!(start, finish);
        }
    }
}
