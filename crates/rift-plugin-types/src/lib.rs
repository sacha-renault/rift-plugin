//! Shared types used across the plugin crates: audio and note ports, MIDI
//! messages, transport, task queues, and cross-thread buffers.

mod audio_ports;
mod event_pre_process;
mod event_source;
mod midi_message;
mod midi_port;
mod task_queue;
mod tasks_capacity;
mod triple_buffer;

pub mod transport;

pub use audio_ports::{AudioPort, MainAudioPort, PAIR_PORT_ID};
pub use event_pre_process::*;
pub use event_source::EventSource;
pub use midi_message::{MidiMessage, MidiMessageKind};
pub use midi_port::MidiPort;
pub use task_queue::TaskQueue;
pub use tasks_capacity::TaskCapacity;
pub use transport::TransportInfo;
pub use triple_buffer::TripleBuffer;
