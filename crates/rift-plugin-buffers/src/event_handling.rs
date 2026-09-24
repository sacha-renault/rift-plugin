use clack_plugin::events::event_types::ParamValueEvent;
use rift_plugin_types::MidiMessage;

pub enum InputEvent {
    MidiEvent(MidiMessage),
    ParamEvent(ParamValueEvent),
}

pub trait ZipEventConfig {
    const MIDI_EVENT_AUTO_HANDLING: bool;
    const PARAM_EVENT_AUTO_HANDLING: bool;
}
