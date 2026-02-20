use clack_plugin::events::event_types::{
    ParamGestureBeginEvent, ParamGestureEndEvent, ParamValueEvent,
};
use clack_plugin::events::{Pckn, UnknownEvent};
use clack_plugin::utils::{ClapId, Cookie};

#[derive(Debug, Clone, Copy)]
pub enum GuiParamEvent {
    ValueEvent(ParamValueEvent),
    GestureStart(ParamGestureBeginEvent),
    GestureEnd(ParamGestureEndEvent),
}

impl GuiParamEvent {
    pub fn value(param_id: ClapId, value: f64) -> Self {
        let event = ParamValueEvent::new(0, param_id, Pckn::match_all(), value, Cookie::empty());
        Self::ValueEvent(event)
    }

    pub fn value_with_pckn(param_id: ClapId, value: f64, pckn: Pckn) -> Self {
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

impl AsRef<UnknownEvent> for GuiParamEvent {
    fn as_ref(&self) -> &UnknownEvent {
        match self {
            GuiParamEvent::ValueEvent(v) => v.as_ref(),
            GuiParamEvent::GestureStart(v) => v.as_ref(),
            GuiParamEvent::GestureEnd(v) => v.as_ref(),
        }
    }
}
