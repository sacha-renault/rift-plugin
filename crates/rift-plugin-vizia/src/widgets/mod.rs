use rift_plugin_gui::{GuiParamEvent, GuiTasks};
use rift_plugin_params::{ClapId, Param};
use vizia::prelude::*;

use crate::NewSignal;

pub struct ParamData {
    default: f32,
    min: f32,
    max: f32,
    id: ClapId,
    value: Signal<f32>,
}

impl<P> From<&P> for ParamData
where
    P: Param,
{
    fn from(param: &P) -> Self {
        Self {
            default: param.normalize(param.default_plain()),
            min: param.min_value(),
            max: param.max_value(),
            id: param.id(),
            value: Signal::new(param.plain()),
        }
    }
}

impl Model for ParamData {
    fn event(&mut self, _: &mut EventContext, event: &mut Event) {
        event.map(|task, meta| match task {
            &GuiTasks::ParamChanged { id, value } if id == self.id => {
                self.value.update(|v| *v = value);
                meta.consume(); // Id is unique, it will not be useful for anyone else
            }
            _ => {}
        });
    }
}

pub fn knob<'a, P: Param>(cx: &'a mut Context, param: &P) -> Handle<'a, Knob<Signal<f32>>> {
    let data = ParamData::from(param);
    let default = data.default;
    let value = data.value;
    let id = data.id;

    cx.emit(NewSignal(id, value));

    data.build(cx);

    Knob::new(cx, default, value, false)
        .on_mouse_down(param_widget_mouse_left_down(id))
        .on_change(emit_param_change(id, value))
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
            // todo!()
            // emit context menu
            // cx.emit(Entity::root())
        }
    }
}

pub fn emit_param_change<T>(id: ClapId, value: Signal<T>) -> impl Fn(&mut EventContext, T)
where
    T: 'static,
    T: Copy + Into<f32>,
{
    move |cx, new| {
        value.update(|v| *v = new);
        cx.emit_to(Entity::root(), GuiParamEvent::value(id, new.into()));
    }
}
