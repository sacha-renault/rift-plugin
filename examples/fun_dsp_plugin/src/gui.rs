use std::sync::Arc;

use rift_plugin::prelude::*;
use rift_plugin_vizia::prelude::*;
use rift_plugin_vizia::widgets::*;

use crate::params::FunDspParams;

pub fn build(params: Arc<FunDspParams>) -> Box<dyn GuiFactory> {
    vizia_gui(480, 480, move |cx, ctx| {
        VStack::new(cx, |cx| {
            Panel::new(cx, "Osc1", |cx| {
                param_knob(cx, &params.oscillators[0].wt_position);
                param_knob(cx, &params.oscillators[0].gain);
                param_knob(cx, &params.oscillators[0].pan);
            });
            Panel::new(cx, "Adsr", |cx| {
                param_knob(cx, &params.oscillators[0].attack);
                param_knob(cx, &params.oscillators[0].decay);
                param_knob(cx, &params.oscillators[0].sustain);
                param_knob(cx, &params.oscillators[0].release);
            });

            Panel::new(cx, "filters", |cx| {
                for fparam in params.filters.iter() {
                    param_knob(cx, &fparam.cutoff);
                    param_selector(cx, &fparam.kind);
                }
            });

            Button::label(cx, "Click for context menu".to_string()).on_press(move |cx| {
                ctx.global_context_menu(cx.mouse().cursor_x as i32, cx.mouse().cursor_y as i32, 0)
            });
        });
    })
}
