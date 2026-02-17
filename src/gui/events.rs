use clack_plugin::events::event_types::{
    ParamGestureBeginEvent, ParamGestureEndEvent, ParamValueEvent,
};
use clack_plugin::events::{Pckn, UnknownEvent};
use clack_plugin::utils::{ClapId, Cookie};

#[derive(Debug)]
pub enum ParamGuiEvent {
    ValueEvent(ParamValueEvent),
    GestureStart(ParamGestureBeginEvent),
    GestureEnd(ParamGestureEndEvent),
}

impl ParamGuiEvent {
    pub fn value(param_id: ClapId, pckn: Pckn, value: f64) -> Self {
        let event = ParamValueEvent::new(0, param_id, pckn, value, Cookie::empty());
        Self::ValueEvent(event)
    }

    pub fn gesture_start(param_id: ClapId) -> Self {
        let event = ParamGestureBeginEvent::new(0, param_id);
        Self::GestureStart(event)
    }

    pub fn gesture_end(param_id: ClapId) -> Self {
        let event = ParamGestureEndEvent::new(0, param_id);
        Self::GestureEnd(event)
    }
}

impl AsRef<UnknownEvent> for ParamGuiEvent {
    fn as_ref(&self) -> &UnknownEvent {
        match self {
            ParamGuiEvent::ValueEvent(v) => v.as_ref(),
            ParamGuiEvent::GestureStart(v) => v.as_ref(),
            ParamGuiEvent::GestureEnd(v) => v.as_ref(),
        }
    }
}
