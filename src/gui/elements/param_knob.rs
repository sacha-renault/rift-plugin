use std::sync::Arc;

use super::gui_prelude::*;

#[derive(ParamBuilder)]
pub struct ParamKnob<L, MapFn> {
    lens: L,
    accessor: MapFn,

    on_value_changed: Option<Arc<dyn Fn(&mut EventContext, f32)>>,
    on_mouse_down: Option<Arc<dyn Fn(&mut EventContext, MouseButton) + Send + Sync>>,
    on_mouse_up: Option<Arc<dyn Fn(&mut EventContext, MouseButton) + Send + Sync>>,
    text_formater: Option<fn(f64, &str) -> String>,
}

impl<L, MapFn, P> ParamKnob<L, MapFn>
where
    P: Clone,
    L: Lens<Target = P> + Copy,
    MapFn: (Fn(&P) -> &dyn ClapParam) + Copy + 'static,
{
    pub fn build_view(self, cx: &mut Context) -> Handle<'_, impl View> {
        let Self {
            lens,
            accessor,
            on_value_changed,
            on_mouse_down,
            on_mouse_up,
            text_formater: text_formatter,
            ..
        } = self;

        let param_ptr = lens.map(move |ps| accessor(ps).as_ptr()).get(cx);
        let value_lens = make_lens(lens, accessor, |p| p.get_raw() as f32);
        let text_lens = make_lens(lens, accessor, move |p| {
            if let Some(f) = text_formatter {
                f(p.get_raw(), p.unit())
            } else {
                format!("{:.2}{}", p.get_raw(), p.unit())
            }
        });
        let default_value = param_ptr.default_raw();

        VStack::new(cx, move |cx| {
            Label::new(cx, text_lens);

            Knob::new(cx, default_value as f32, value_lens, false)
                .height(Units::Percentage(100.))
                .width(Units::Percentage(100.))
                .on_change(move |cx, v| {
                    set_value_normalized(param_ptr, cx, v as f64);
                    on_value_changed.as_ref().map(|f| f(cx, v));
                })
                .on_mouse_down(move |cx, mb| {
                    gesture_start(param_ptr, cx);
                    on_mouse_down.as_ref().map(|f| f(cx, mb));
                })
                .on_mouse_up(move |cx, mb| {
                    gesture_end(param_ptr, cx);
                    on_mouse_up.as_ref().map(|f| f(cx, mb));
                });
        })
        .width(Units::Pixels(50.))
        .height(Units::Pixels(50.))
    }
}
