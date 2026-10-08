use crate::dev_prelude::*;

pub fn knob<'a, P: Param>(cx: &'a mut Context, param: &P) -> Handle<'a, Knob<Signal<f32>>> {
    let binding = ParamBinding::new(cx, param);

    Knob::new(
        cx,
        binding.default_normalized(),
        binding.normalized(),
        false,
    )
    .on_mouse_down(param_widget_mouse_left_down(binding))
    .on_change(emit_param_change(binding))
    .on_mouse_up(param_widget_mouse_left_up(binding))
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
