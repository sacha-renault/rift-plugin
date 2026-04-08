use clack_plugin::{
    events::{
        Pckn, UnknownEvent,
        event_types::{ParamGestureBeginEvent, ParamGestureEndEvent, ParamValueEvent},
    },
    utils::{ClapId, Cookie},
};

#[derive(Debug, Clone, Copy)]
pub enum GuiParamEventKind {
    /// A parameter value change event.
    Value(f32),
    /// Start of a mouse gesture interaction.
    GestureBegin,
    /// End of a mouse gesture interaction.
    GestureEnd,
    /// This allow the audio thread the a value as changed
    /// without a f64 value.
    ValueLess,
}

/// The GUI-side event wrapper passed to `GuiView::handle_event` for parameter interactions.
#[derive(Debug, Clone, Copy)]
pub struct GuiParamEvent {
    pub param_id: ClapId,
    pub kind: GuiParamEventKind,
}

impl GuiParamEvent {
    /// Creates an event for a parameter value change.
    pub fn value(param_id: ClapId, value: f32) -> Self {
        Self {
            param_id,
            kind: GuiParamEventKind::Value(value),
        }
    }

    /// Creates an event signaling the start of a gesture interaction.
    pub fn gesture_start(param_id: ClapId) -> Self {
        Self {
            param_id,
            kind: GuiParamEventKind::GestureBegin,
        }
    }

    /// Creates an event signaling the end of a gesture interaction.
    pub fn gesture_end(param_id: ClapId) -> Self {
        Self {
            param_id,
            kind: GuiParamEventKind::GestureEnd,
        }
    }

    pub fn value_less(param_id: ClapId) -> Self {
        Self {
            param_id,
            kind: GuiParamEventKind::ValueLess,
        }
    }

    /// Converts this GUI event into the raw type expected by the host plugin.
    ///
    /// # Note
    /// This creates new instances wrapping the original event ID and value.
    pub fn maybe_to_raw(&self) -> Option<RawParamEvent> {
        let raw = match self.kind {
            GuiParamEventKind::Value(v) => RawParamEvent::Value(ParamValueEvent::new(
                0,
                self.param_id,
                Pckn::match_all(),
                v as f64,
                Cookie::empty(),
            )),
            GuiParamEventKind::GestureBegin => {
                RawParamEvent::GestureBegin(ParamGestureBeginEvent::new(0, self.param_id))
            }
            GuiParamEventKind::GestureEnd => {
                RawParamEvent::GestureEnd(ParamGestureEndEvent::new(0, self.param_id))
            }
            _ => return None,
        };

        Some(raw)
    }
}

/// Event to be sent through internal messagine in
/// [`crate::wrapper::shared_states::PluginSharedState`]
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

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u32) -> ClapId {
        ClapId::from(n)
    }

    #[test]
    fn test_value_event() {
        let e = GuiParamEvent::value(id(1), 0.75);
        assert_eq!(e.param_id, id(1));
        assert!(matches!(e.kind, GuiParamEventKind::Value(v) if v == 0.75));
    }

    #[test]
    fn test_gesture_start() {
        let e = GuiParamEvent::gesture_start(id(2));
        assert_eq!(e.param_id, id(2));
        assert!(matches!(e.kind, GuiParamEventKind::GestureBegin));
    }

    #[test]
    fn test_gesture_end() {
        let e = GuiParamEvent::gesture_end(id(3));
        assert_eq!(e.param_id, id(3));
        assert!(matches!(e.kind, GuiParamEventKind::GestureEnd));
    }

    #[test]
    fn test_to_raw_value() {
        let e = GuiParamEvent::value(id(1), 0.5);
        assert!(matches!(e.maybe_to_raw(), Some(RawParamEvent::Value(_))));
    }

    #[test]
    fn test_to_raw_gesture_begin() {
        let e = GuiParamEvent::gesture_start(id(1));
        assert!(matches!(
            e.maybe_to_raw(),
            Some(RawParamEvent::GestureBegin(_))
        ));
    }

    #[test]
    fn test_to_raw_gesture_end() {
        let e = GuiParamEvent::gesture_end(id(1));
        assert!(matches!(
            e.maybe_to_raw(),
            Some(RawParamEvent::GestureEnd(_))
        ));
    }

    #[test]
    fn test_raw_as_ref_unknown_event() {
        // just verifying AsRef doesn't panic for each variant
        let events = [
            GuiParamEvent::value(id(1), 1.0),
            GuiParamEvent::gesture_start(id(1)),
            GuiParamEvent::gesture_end(id(1)),
        ];
        for e in events {
            let _: &UnknownEvent = e.maybe_to_raw().unwrap().as_ref();
        }
    }

    #[test]
    fn test_copy_clone() {
        let e = GuiParamEvent::value(id(42), 0.3);
        let e2 = e;
        assert!(matches!(e2.kind, GuiParamEventKind::Value(v) if v == 0.3));
    }
}
