use vizia::prelude::*;

use crate::{
    gui::GuiParamEvent,
    prelude::{ClapParam, ParamPtr},
};

pub fn make_lens<L, P, MapFn, F, R>(params: L, accessor: MapFn, f: F) -> impl Lens<Target = R>
where
    L: Lens<Target = P> + Copy,
    MapFn: 'static + Copy + Fn(&P) -> &dyn ClapParam,
    F: Fn(&dyn ClapParam) -> R + Clone + 'static,
    R: Clone + 'static,
{
    params.map(move |params| {
        let param = accessor(params);
        f(param)
    })
}

pub fn gesture_start(param_ptr: ParamPtr, cx: &mut EventContext) {
    cx.emit(GuiParamEvent::gesture_start(param_ptr.id()));
}

pub fn gesture_end(param_ptr: ParamPtr, cx: &mut EventContext) {
    cx.emit(GuiParamEvent::gesture_end(param_ptr.id()));
}

pub fn set_value(param_ptr: ParamPtr, cx: &mut EventContext, value: f64) {
    cx.emit(GuiParamEvent::value(param_ptr.id(), value));
}

pub fn set_value_normalized(param_ptr: ParamPtr, cx: &mut EventContext, value: f64) {
    let normalized = param_ptr.normalize(value);
    cx.emit(GuiParamEvent::value(param_ptr.id(), normalized));
}
