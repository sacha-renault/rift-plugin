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

pub trait ParamBinding {
    // pub fn from_lens<P, MapFn>(
    //     cx: &mut Context,
    //     lens: impl Lens<Target = P> + Copy,
    //     accessor: MapFn,
    // ) -> Self
    // where
    //     P: Clone,
    //     MapFn: 'static + Copy + Fn(&P) -> &dyn ClapParam,
    // {
    //     let param_ptr = lens.map(move |p| accessor(p).as_ptr()).get(cx);
    //     Self::new(param_ptr)
    // }

    // pub fn new(param_ptr: ParamPtr) -> Self {
    //     Self { param_ptr }
    // }

    fn gesture_start(&self, cx: &mut Context) {
        cx.emit(GuiParamEvent::gesture_start(self.param_ptr().id()));
    }

    fn gesture_end(&self, cx: &mut Context) {
        cx.emit(GuiParamEvent::gesture_end(self.param_ptr().id()));
    }

    fn set_value(&self, cx: &mut Context, value: f64) {
        cx.emit(GuiParamEvent::value(self.param_ptr().id(), value));
    }

    fn set_value_normalized(&self, cx: &mut Context, value: f64) {
        let normalized = self.param_ptr().normalize(value);
        cx.emit(GuiParamEvent::value(self.param_ptr().id(), normalized));
    }

    fn param_ptr(&self) -> ParamPtr;
}
