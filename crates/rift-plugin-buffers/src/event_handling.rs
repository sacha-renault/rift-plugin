use clack_plugin::events::UnknownEvent;
use clack_plugin::events::event_types::{MidiEvent, ParamValueEvent};
use rift_plugin_types::MidiMessage;

#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    MidiEvent(MidiMessage),
    ParamEvent(ParamValueEvent),
}

impl TryFrom<&UnknownEvent> for InputEvent {
    // Unknown variant
    type Error = ();

    fn try_from(event: &UnknownEvent) -> Result<Self, Self::Error> {
        if let Some(&p) = event.as_event::<ParamValueEvent>() {
            Ok(InputEvent::ParamEvent(p))
        } else if let Some(&m) = event.as_event::<MidiEvent>() {
            Ok(InputEvent::MidiEvent(m.into()))
        } else {
            Err(())
        }
    }
}
