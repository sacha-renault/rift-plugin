use std::sync::Arc;

use crate::prelude::ParamPtr;

use super::gui_prelude::*;

#[derive(Lens)]
struct DataXY {
    param_ptr_x: ParamPtr,
    param_ptr_y: ParamPtr,
    xy: (f32, f32),
}

impl Model for DataXY {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|&SetValueXY(x, y), _| {
            if self.xy.0 != x {
                self.xy.0 = x;
                set_value_normalized(self.param_ptr_x, cx, x as f64);
            }

            if self.xy.1 != y {
                self.xy.1 = y;
                set_value_normalized(self.param_ptr_y, cx, y as f64);
            }
        });
    }
}

struct SetValueXY(f32, f32);

/// A control for mapping a [`ClapParam`] to a rotary knob UI element.
#[derive(ParamViewBuilder)]
pub struct ParamPadXY<L, MapFnX, MapFnY>
where
    L: Lens + Copy,
    L::Target: Clone,
    MapFnX: (Fn(&L::Target) -> &dyn ClapParam) + Copy + 'static,
    MapFnY: (Fn(&L::Target) -> &dyn ClapParam) + Copy + 'static,
{
    #[builder(new)]
    lens: L,

    #[builder(new)]
    accessor_x: MapFnX,

    #[builder(new)]
    accessor_y: MapFnY,

    on_value_changed: Option<Arc<dyn Fn(&mut EventContext, f32, f32)>>,
    on_mouse_down: Option<Arc<dyn Fn(&mut EventContext, MouseButton) + Send + Sync>>,
    on_mouse_up: Option<Arc<dyn Fn(&mut EventContext, MouseButton) + Send + Sync>>,
    value_text_formater: Option<fn(f64, &str) -> String>,

    /// A function that will be called on the [`Handle<'_, Knob<L>>`]. It allow to modify
    /// the [`Handle`] as [`LayoutModifiers`] and [`StyleModifiers`]
    #[builder(default = None)]
    pad_modifier: Option<Arc<dyn Fn(Handle<'_, FView>) -> Handle<'_, FView>>>,
}

impl<L, MapFnX, MapFnY> ParamPadXY<L, MapFnX, MapFnY>
where
    L: Lens + Copy,
    L::Target: Clone,
    MapFnX: (Fn(&L::Target) -> &dyn ClapParam) + Copy + 'static,
    MapFnY: (Fn(&L::Target) -> &dyn ClapParam) + Copy + 'static,
{
    pub fn build_view(self, cx: &mut Context) -> Handle<'_, impl View> {
        let Self {
            lens,
            accessor_x,
            accessor_y,
            on_value_changed,
            on_mouse_down,
            on_mouse_up,
            value_text_formater,
            pad_modifier,
        } = self;

        let param_ptr_x = lens.map(move |ps| accessor_x(ps).as_ptr()).get(cx);
        let param_ptr_y = lens.map(move |ps| accessor_y(ps).as_ptr()).get(cx);
        let xy = (
            param_ptr_x.get_normalized() as f32,
            param_ptr_y.get_normalized() as f32,
        );

        DataXY {
            param_ptr_x,
            param_ptr_y,
            xy,
        }
        .build(cx);

        // let text_lens = make_lens(lens, accessor, move |p| {
        //     if let Some(f) = value_text_formater {
        //         f(p.get_raw(), p.unit())
        //     } else {
        //         format!("{:.2}{}", p.get_raw(), p.unit())
        //     }
        // });
        // let default_value = param_ptr.normalize(param_ptr.default_raw());

        VStack::new(cx, move |cx| {
            // Label::new(cx, text_lens)
            //     .maybe_apply_modifiers(label_text_modifier.as_deref())
            //     .class("knob-value-label");

            XYPad::new(cx, DataXY::xy)
                .on_change(move |cx, value_x, value_y| {
                    cx.emit(SetValueXY(value_x, value_y));
                    if let Some(f) = on_value_changed.as_ref() {
                        f(cx, value_x, value_y)
                    }
                })
                .on_mouse_down(move |cx, mb| {
                    gesture_start(param_ptr_x, cx);
                    gesture_start(param_ptr_y, cx);
                    if let Some(f) = on_mouse_down.as_ref() {
                        f(cx, mb)
                    }
                })
                .on_mouse_up(move |cx, mb| {
                    gesture_end(param_ptr_x, cx);
                    gesture_end(param_ptr_y, cx);
                    if let Some(f) = on_mouse_up.as_ref() {
                        f(cx, mb)
                    }
                })
                .maybe_apply_modifiers(pad_modifier.as_deref())
                .class("xy-pad");
        })
        .class("xy-pad-container")
    }
}
