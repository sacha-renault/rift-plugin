use rift_plugin_gui::GuiTasks;
use rift_plugin_params::{ClapId, Param};
use vizia::prelude::*;

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
        event.map(|task, ev_cx| match task {
            &GuiTasks::ParamChanged { id, value } if id == self.id => {
                self.value.update(|v| *v = value);
                ev_cx.consume(); // Id is unique, it will not be useful for anyone else
            }
            _ => {}
        });
    }
}

pub fn knob<'a, P: Param>(cx: &'a mut Context, param: &P) -> Handle<'a, Knob<Signal<f32>>> {
    let data = ParamData::from(param);
    let default = data.default;
    let value = data.value;
    data.build(cx);

    Knob::new(cx, default, value, false).on_change(move |_, new| value.update(|v| *v = new))
}
