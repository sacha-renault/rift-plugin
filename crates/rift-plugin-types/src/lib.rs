//! Declarative CLAP-facing types shared across the plugin crates: audio and
//! note ports, MIDI messages, and the event source marker.

mod audio_ports;
mod event_source;
mod midi_message;
mod midi_port;

pub use audio_ports::{AudioPort, MainAudioPort, PAIR_PORT_ID};
pub use event_source::EventSource;
pub use midi_message::{MidiMessage, MidiMessageKind};
pub use midi_port::MidiPort;
