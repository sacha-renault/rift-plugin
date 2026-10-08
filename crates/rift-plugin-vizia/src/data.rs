use std::collections::HashMap;

use rift_plugin_params::{ClapId, Param, ParamPtr};
use vizia::prelude::*;

#[derive(Clone)]
pub struct ParamData {
    pub(crate) ptr: ParamPtr,
    pub(crate) signal: Signal<f32>,
}

pub struct ParamSignals {
    pub signals: HashMap<ClapId, ParamData>,
}

pub struct NewSignal(ParamPtr, Signal<f32>);

/// TODO, this doesn't currently work, since all registeration occures in a single frame
/// reg doesn't contain the clap id yet when a second widget is trying to register.
pub fn register_param<P: Param>(cx: &mut Context, param: &P) -> Signal<f32> {
    let signal = Signal::new(param.normalized());
    cx.emit(NewSignal(param.as_ptr(), signal));
    signal
}

impl Model for ParamSignals {
    fn event(&mut self, _: &mut vizia::prelude::EventContext, event: &mut Event) {
        event.map(|&NewSignal(ptr, signal): &NewSignal, meta| {
            self.signals.insert(ptr.id(), ParamData { ptr, signal });
            meta.consume();
        });
    }
}
