use std::sync::Arc;

use rift_plugin_core::params::FloatParam;

use crate::PopupExt;

use super::gui_prelude::*;

/// A control for mapping a [`ClapParam`] to a rotary knob UI element.
#[derive(ParamViewBuilder)]
pub struct ParamKnob<L, MapFn>
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

    /// A function that will be called on the [`Handle<'_, Knob<L>>`]. It allow to modify
    /// the [`Handle`] as [`LayoutModifiers`] and [`StyleModifiers`]
    #[builder(default = None)]
    knob_modifiers: Option<Arc<dyn Fn(Handle<'_, FView>) -> Handle<'_, FView>>>,
    label_text_modifier: Option<Arc<dyn Fn(Handle<'_, FView>) -> Handle<'_, FView>>>,
    label_name_modifier: Option<Arc<dyn Fn(Handle<'_, FView>) -> Handle<'_, FView>>>,
    arctrack_modifier: Option<Arc<dyn Fn(Handle<'_, FView>) -> Handle<'_, FView>>>,
    tick_modifier: Option<Arc<dyn Fn(Handle<'_, FView>) -> Handle<'_, FView>>>,

    /// Range of the knob (start, end)
    ///
    /// Ensure end - start < 360. to avoid weird behavior
    #[builder(default = (-240., 60.))]
    knob_range: (f32, f32),

    #[builder(default = true)]
    has_name_label: bool,

    #[builder(default = 15.)]
    span: f32,

    #[builder(default = false)]
    centered: bool,
}

impl<L, MapFn> DestructThenBuildView for ParamKnob<L, MapFn>
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
            knob_modifiers,
            label_text_modifier,
            label_name_modifier,
            arctrack_modifier,
            tick_modifier,
            knob_range: (start_angle, end_angle),
            has_name_label,
            span,
            centered,
        } = self;

        let sweep = end_angle - start_angle;
        let offset = sweep / 2.0;

        let param_ptr = lens.map(move |ps| accessor(ps).as_ptr()).get(cx);
        let value_lens = make_lens(lens, accessor, move |p| p.get_normalized());

        let text_lens = make_lens(lens, accessor, move |p| {
            if let Some(f) = value_text_formater {
                f(p.get_raw(), p.unit())
            } else {
                format!("{:.2}{}", p.get_raw(), p.unit())
            }
        });
        let default_value = param_ptr.normalize(param_ptr.default_raw());

        VStack::new(cx, move |cx| {
            ActiveElementData::new(CursorIcon::RowResize).build(cx);

            if has_name_label {
                Label::new(cx, param_ptr.name())
                    .maybe_apply_modifiers(label_name_modifier.as_deref())
                    .class("knob-name-label");
            }

            Knob::custom(cx, default_value, value_lens, move |cx, lens| {
                ZStack::new(cx, |cx| {
                    ArcTrack::new(
                        cx,
                        centered,
                        Percentage(100.0),
                        Percentage(span),
                        start_angle,
                        end_angle,
                        KnobMode::Continuous,
                    )
                    .value(lens)
                    .maybe_apply_modifiers(arctrack_modifier.as_deref())
                    .class("knob-track");

                    HStack::new(cx, |cx| {
                        Element::new(cx).class("knob-tick");
                    })
                    .maybe_apply_modifiers(tick_modifier.as_deref())
                    .rotate(lens.map(move |v| Angle::Deg(*v * sweep - offset)))
                    .on_hover(|cx| cx.emit(ActiveElementEvent::StartHover))
                    .on_hover_out(|cx| cx.emit(ActiveElementEvent::EndHover))
                    .class("knob-head");
                })
            })
            .on_change(move |cx, v| {
                set_value_normalized(param_ptr, cx, v);
                if let Some(f) = on_value_changed.as_ref() {
                    f(cx, v)
                }
            })
            .on_mouse_down(move |cx, mb| {
                if mb == MouseButton::Left {
                    gesture_start(param_ptr, cx);
                    cx.emit(ActiveElementEvent::StartDragging);
                    if let Some(f) = on_mouse_down.as_ref() {
                        f(cx, mb)
                    }
                }
            })
            .on_mouse_up(move |cx, mb| match mb {
                MouseButton::Left => {
                    gesture_end(param_ptr, cx);
                    cx.emit(ActiveElementEvent::EndDragging);
                    if let Some(f) = on_mouse_up.as_ref() {
                        f(cx, mb)
                    }
                }
                MouseButton::Right => cx.emit(ContextMenuEvent(param_ptr.id())),
                _ => {}
            })
            .maybe_apply_modifiers(knob_modifiers.as_deref())
            .popup_on(
                move |cx| {
                    Label::new(cx, text_lens)
                        .maybe_apply_modifiers(label_text_modifier.as_deref())
                        .class("knob-value-label");
                },
                ActiveElementData::is_active(),
            )
            .class("knob");

            // Label::new(cx, text_lens)
            //     .maybe_apply_modifiers(label_text_modifier.as_deref())
            //     .class("knob-value-label");
        })
        .class("knob-container")
    }
}
