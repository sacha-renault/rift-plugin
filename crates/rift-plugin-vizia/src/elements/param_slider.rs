use std::sync::Arc;

use rift_plugin_core::params::FloatParam;

use super::gui_prelude::*;

/// A control for mapping a [`ClapParam`] to a Slider.
#[derive(ParamViewBuilder)]
pub struct ParamSlider<L, MapFn>
where
    L: Lens + Copy,
    L::Target: Clone,
    MapFn: (Fn(&L::Target) -> &FloatParam) + Copy + 'static,
{
    #[builder(new)]
    lens: L,

    #[builder(new)]
    accessor: MapFn,

    on_value_changed: Option<Arc<dyn Fn(&mut EventContext, f32)>>,
    on_mouse_down: Option<Arc<dyn Fn(&mut EventContext, MouseButton) + Send + Sync>>,
    on_mouse_up: Option<Arc<dyn Fn(&mut EventContext, MouseButton) + Send + Sync>>,
    value_text_formater: Option<fn(f32, &str) -> String>,

    /// A function that will be called on the [`Handle<'_, Slider<L>>`]. It allow to modify
    /// the [`Handle`] as [`LayoutModifiers`] and [`StyleModifiers`]
    #[builder(default = None)]
    slider_modifier: Option<Arc<dyn Fn(Handle<'_, FView>) -> Handle<'_, FView>>>,

    #[builder(default = None)]
    label_text_modifier: Option<Arc<dyn Fn(Handle<'_, FView>) -> Handle<'_, FView>>>,

    #[builder(default = 1e-6)]
    step: f32,

    #[builder(default = Orientation::Vertical)]
    orientation: Orientation,
}

impl<L, MapFn> DestructThenBuildView for ParamSlider<L, MapFn>
where
    L: Lens + Copy,
    L::Target: Clone,
    MapFn: (Fn(&L::Target) -> &FloatParam) + Copy + 'static,
{
    fn build_view(self, cx: &mut Context) -> Handle<'_, impl View> {
        let Self {
            lens,
            accessor,
            on_value_changed,
            on_mouse_down,
            on_mouse_up,
            value_text_formater,
            slider_modifier,
            label_text_modifier,
            step,
            orientation,
        } = self;

        let param_ptr = lens.map(move |ps| accessor(ps).as_ptr()).get(cx);
        let value_lens = make_lens(lens, accessor, move |p| p.get_normalized());
        let (start, end) = (param_ptr.min_value(), param_ptr.max_value());
        let text_lens = make_lens(lens, accessor, move |p| {
            if let Some(f) = value_text_formater {
                f(p.get_raw(), p.unit())
            } else {
                format!("{:.2}{}", p.get_raw(), p.unit())
            }
        });

        let mut handle =
            Element::new(cx)
                .class("slider-container")
                .layout_type(match orientation {
                    Orientation::Horizontal => LayoutType::Row,
                    Orientation::Vertical => LayoutType::Column,
                });

        let entity = handle.entity();
        handle.context().with_current(entity, move |cx| {
            Label::new(cx, text_lens)
                .maybe_apply_modifiers(label_text_modifier.as_deref())
                .class("slider-value-label");

            Slider::new(cx, value_lens)
                .step(step)
                .range(start..end)
                .orientation(orientation)
                .on_change(move |cx, v| {
                    set_value_normalized(param_ptr, cx, v);
                    if let Some(f) = on_value_changed.as_ref() {
                        f(cx, v)
                    }
                })
                .on_mouse_down(move |cx, mb| {
                    if mb == MouseButton::Left {
                        gesture_start(param_ptr, cx);
                        if let Some(f) = on_mouse_down.as_ref() {
                            f(cx, mb)
                        }
                    }
                })
                .on_mouse_up(move |cx, mb| match mb {
                    MouseButton::Left => {
                        gesture_end(param_ptr, cx);
                        if let Some(f) = on_mouse_up.as_ref() {
                            f(cx, mb)
                        }
                    }
                    MouseButton::Right => cx.emit(ContextMenuEvent(param_ptr.id())),
                    _ => {}
                })
                .maybe_apply_modifiers(slider_modifier.as_deref())
                .class("slider");
        });
        handle
    }
}
