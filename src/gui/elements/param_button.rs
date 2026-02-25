use std::sync::Arc;

use super::gui_prelude::*;

#[derive(ParamViewBuilder)]
pub struct ParamButton<L, MapFn>
where
    L: 'static,
    MapFn: 'static,
{
    lens: L,
    accessor: MapFn,

    on_press: Option<Arc<dyn Fn(&mut EventContext, f32) + Send + Sync>>,

    /// A function that will be called on the [`Handle<'_, Knob<L>>`]. It allow to modify
    /// the [`Handle`] as [`LayoutModifiers`] and [`StyleModifiers`]
    #[builder(default = None)]
    button_modifiers: Option<Arc<dyn Fn(Handle<'_, FView>) -> Handle<'_, FView>>>,
}

impl<L, MapFn> ParamButton<L, MapFn>
where
    L: Lens + Copy,
    L::Target: Clone,
    MapFn: (Fn(&L::Target) -> &dyn ClapParam) + Copy + 'static,
{
    pub fn build_view(self, cx: &mut Context) -> Handle<'_, impl View> {
        let Self {
            lens,
            accessor,
            on_press,
            button_modifiers,
        } = self;

        let param_ptr = lens.map(move |ps| accessor(ps).as_ptr()).get(cx);
        let value_lens = make_lens(lens, accessor, |p| p.get_normalized() > 0.5);

        HStack::new(cx, move |cx| {
            Button::new(cx, |cx| Label::new(cx, param_ptr.name()))
                .toggle_class("accent", value_lens)
                .on_press(move |cx| {
                    let new_value = if param_ptr.get_normalized() > 0.5 {
                        0.0
                    } else {
                        1.0
                    };

                    gesture_start(param_ptr, cx);
                    set_value_normalized(param_ptr, cx, new_value);
                    gesture_end(param_ptr, cx);
                    on_press.as_ref().map(|f| f(cx, new_value as f32));
                })
                .maybe_apply_modifiers(button_modifiers.as_deref())
                .class("button");
        })
        .class("button-container")
    }
}
