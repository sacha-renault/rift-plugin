use clack_plugin::events::event_types::{
    ParamGestureBeginEvent, ParamGestureEndEvent, ParamValueEvent,
};
use clack_plugin::events::{Pckn, UnknownEvent};
use clack_plugin::utils::{ClapId, Cookie};

#[derive(Debug, Clone, Copy)]
pub enum GuiParamEventKind {
    Value(f64),
    GestureBegin,
    GestureEnd,
}

#[derive(Debug, Clone, Copy)]
pub struct GuiParamEvent {
    pub(crate) param_id: ClapId,
    pub(crate) kind: GuiParamEventKind,
}

impl GuiParamEvent {
    pub fn value(param_id: ClapId, value: f64) -> Self {
        Self {
            param_id,
            kind: GuiParamEventKind::Value(value),
        }
    }

    pub fn gesture_start(param_id: ClapId) -> Self {
        Self {
            param_id,
            kind: GuiParamEventKind::GestureBegin,
        }
    }

    pub fn gesture_end(param_id: ClapId) -> Self {
        Self {
            param_id,
            kind: GuiParamEventKind::GestureEnd,
        }
    }

    pub fn to_raw(&self) -> RawParamEvent {
        match self.kind {
            GuiParamEventKind::Value(v) => RawParamEvent::Value(ParamValueEvent::new(
                0,
                self.param_id,
                Pckn::match_all(),
                v,
                Cookie::empty(),
            )),
            GuiParamEventKind::GestureBegin => {
                RawParamEvent::GestureBegin(ParamGestureBeginEvent::new(0, self.param_id))
            }
            GuiParamEventKind::GestureEnd => {
                RawParamEvent::GestureEnd(ParamGestureEndEvent::new(0, self.param_id))
            }
        }
    }
}

pub enum RawParamEvent {
    Value(ParamValueEvent),
    GestureBegin(ParamGestureBeginEvent),
    GestureEnd(ParamGestureEndEvent),
}

impl AsRef<UnknownEvent> for RawParamEvent {
    fn as_ref(&self) -> &UnknownEvent {
        match self {
            RawParamEvent::Value(e) => e.as_ref(),
            RawParamEvent::GestureBegin(e) => e.as_ref(),
            RawParamEvent::GestureEnd(e) => e.as_ref(),
        }
    }
}
