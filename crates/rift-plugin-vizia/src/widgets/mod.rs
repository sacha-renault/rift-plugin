use crate::dev_prelude::*;

mod dial;

pub use dial::{DialExt, DialModifiers};

pub fn param_knob<'a, P: Param>(cx: &'a mut Context, param: &P) -> Handle<'a, DialModifiers> {
    let binding = ParamBinding::new(cx, param);
    let dial = DialModifiers::labeled(cx, binding.name(), binding.text(), binding.normalized());
    binding.connect(dial)
}

pub fn param_widget_mouse_left_down(
    binding: ParamBinding,
) -> impl Fn(&mut EventContext, MouseButton) {
    move |cx, btn| {
        if btn.eq(&MouseButton::Left) {
            binding.begin(cx);
        }
    }
}

pub fn param_widget_mouse_left_up(
    binding: ParamBinding,
) -> impl Fn(&mut EventContext, MouseButton) {
    move |cx, btn| {
        if btn.eq(&MouseButton::Left) {
            binding.end(cx);
        } else if btn.eq(&MouseButton::Right) {
            binding.open_context_menu(cx);
        }
    }
}

pub fn emit_param_change(binding: ParamBinding) -> impl Fn(&mut EventContext, f32) {
    move |cx, normalized| binding.set(cx, normalized)
}
