use crate::dev_prelude::*;

pub fn knob<'a, P: Param>(cx: &'a mut Context, param: &P) -> Handle<'a, Knob<Signal<f32>>> {
    let signal = register_param(cx, param);
    let id = param.id();
    let default = param.normalize(param.default_plain());

    Knob::new(cx, default, signal, false)
        .on_mouse_down(param_widget_mouse_left_down(id))
        .on_change(emit_param_change(id, param.as_ptr(), signal))
        .on_mouse_up(param_widget_mouse_left_up(id))
}

pub fn param_widget_mouse_left_down(id: ClapId) -> impl Fn(&mut EventContext, MouseButton) {
    move |cx, btn| {
        if btn.eq(&MouseButton::Left) {
            cx.emit_to(Entity::root(), GuiParamEvent::gesture_start(id));
        }
    }
}

pub fn param_widget_mouse_left_up(id: ClapId) -> impl Fn(&mut EventContext, MouseButton) {
    move |cx, btn| {
        if btn.eq(&MouseButton::Left) {
            cx.emit_to(Entity::root(), GuiParamEvent::gesture_end(id));
        } else if btn.eq(&MouseButton::Right) {
            let scale = cx.scale_factor().max(f32::EPSILON);
            let mouse = cx.mouse();
            let request = ParamContextMenuRequest {
                id,
                x: (mouse.cursor_x / scale).round() as i32,
                y: (mouse.cursor_y / scale).round() as i32,
                screen: 0,
            };
            cx.emit_to(Entity::root(), request);
        }
    }
}

pub fn emit_param_change(
    id: ClapId,
    ptr: ParamPtr,
    value: Signal<f32>,
) -> impl Fn(&mut EventContext, f32) {
    move |cx, new| {
        value.update(|v| *v = new);
        cx.emit_to(
            Entity::root(),
            GuiParamEvent::value(id, ptr.denormalize(new)),
        );
    }
}
