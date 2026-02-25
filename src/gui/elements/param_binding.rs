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
    let normalized = param_ptr.denormalize(value);
    cx.emit(GuiParamEvent::value(param_ptr.id(), normalized));
}

pub fn apply_opt_binding<E, F, T>(element: E, opt_value: Option<T>, func: F) -> E
where
    F: FnOnce(E, T) -> E,
{
    if let Some(value) = opt_value {
        func(element, value)
    } else {
        element
    }
}

pub struct FView;

impl View for FView {}

pub type ModifierFn = dyn Fn(Handle<'_, FView>) -> Handle<'_, FView>;

pub(crate) trait ViewApplyModifiers<'a>: Sized {
    fn maybe_apply_modifiers<F>(self, func: Option<F>) -> Handle<'a, FView>
    where
        F: Fn(Handle<'a, FView>) -> Handle<'a, FView>;
}

impl<'a, T> ViewApplyModifiers<'a> for Handle<'a, T>
where
    T: View,
{
    fn maybe_apply_modifiers<F>(self, func: Option<F>) -> Handle<'a, FView>
    where
        F: Fn(Handle<'a, FView>) -> Handle<'a, FView>,
    {
        // SAFETY: Handle<T> and Handle<FView> are identical in layout,
        // PhantomData<T> is a ZST. We're just rebranding the type tag.
        let handle: Handle<'a, FView> = unsafe { std::mem::transmute(self) };

        if let Some(f) = func {
            f(handle)
        } else {
            handle
        }
    }
}
