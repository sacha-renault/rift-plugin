use std::sync::Arc;

use super::gui_prelude::*;

#[derive(ParamViewBuilder)]
pub struct ParamKnob<L, MapFn>
where
    L: 'static,
    MapFn: 'static,
{
    lens: L,
    accessor: MapFn,

    on_value_changed: Option<Arc<dyn Fn(&mut EventContext, f32)>>,
    on_mouse_down: Option<Arc<dyn Fn(&mut EventContext, MouseButton) + Send + Sync>>,
    on_mouse_up: Option<Arc<dyn Fn(&mut EventContext, MouseButton) + Send + Sync>>,
    value_text_formater: Option<fn(f64, &str) -> String>,

    /// A function that will be called on the [`Handle<'_, Knob<L>>`]. It allow to modify
    /// the [`Handle`] as [`LayoutModifiers`] and [`StyleModifiers`]
    #[builder(default = None)]
    knob_modifiers: Option<Arc<dyn Fn(Handle<'_, FView>) -> Handle<'_, FView>>>,
    label_text_modifier: Option<Arc<dyn Fn(Handle<'_, FView>) -> Handle<'_, FView>>>,
    arctrack_modifier: Option<Arc<dyn Fn(Handle<'_, FView>) -> Handle<'_, FView>>>,
    tick_modifier: Option<Arc<dyn Fn(Handle<'_, FView>) -> Handle<'_, FView>>>,
}

impl<L, MapFn> ParamKnob<L, MapFn>
where
    L: Lens + Copy,
    L::Target: Clone,
    MapFn: (Fn(&L::Target) -> &dyn ClapParam) + Copy + 'static,
{
    pub fn build_view(self, cx: &mut Context) -> Handle<'_, impl View> {
        let Self {
            lens,
            accessor,
            on_value_changed,
            on_mouse_down,
            on_mouse_up,
            value_text_formater,
            knob_modifiers,
            label_text_modifier,
            arctrack_modifier,
            tick_modifier,
        } = self;

        let param_ptr = lens.map(move |ps| accessor(ps).as_ptr()).get(cx);
        let value_lens = make_lens(lens, accessor, |p| p.get_normalized() as f32);
        let text_lens = make_lens(lens, accessor, move |p| {
            if let Some(f) = value_text_formater {
                f(p.get_raw(), p.unit())
            } else {
                format!("{:.2}{}", p.get_raw(), p.unit())
            }
        });
        let default_value = param_ptr.normalize(param_ptr.default_raw());

        VStack::new(cx, move |cx| {
            Label::new(cx, text_lens)
                .maybe_apply_modifiers(label_text_modifier.as_deref())
                .class("knob-value-label");

            Knob::custom(cx, default_value as f32, value_lens, move |cx, lens| {
                ZStack::new(cx, |cx| {
                    ArcTrack::new(
                        cx,
                        false,
                        Percentage(100.0),
                        Percentage(15.0),
                        -240.,
                        60.,
                        KnobMode::Continuous,
                    )
                    .value(lens)
                    .maybe_apply_modifiers(arctrack_modifier.as_deref())
                    .class("knob-track");

                    HStack::new(cx, |cx| {
                        Element::new(cx).class("knob-tick");
                    })
                    .maybe_apply_modifiers(tick_modifier.as_deref())
                    .rotate(lens.map(|v| Angle::Deg(*v * 300.0 - 150.0)))
                    .class("knob-head");
                })
            })
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
            })
            .maybe_apply_modifiers(knob_modifiers.as_deref())
            .class("knob");
        })
        .class("knob-container")
    }
}
