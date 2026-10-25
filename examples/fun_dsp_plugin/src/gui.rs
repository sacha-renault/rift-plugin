use std::sync::Arc;

use rift_plugin::prelude::*;
use rift_plugin_vizia::prelude::*;
use rift_plugin_vizia::widgets::*;

use crate::graph::Shared;
use crate::params::FunDspParams;

pub fn build(params: Arc<FunDspParams>, data: Arc<Shared>) -> Box<dyn GuiFactory> {
    vizia_gui(480, 480, move |cx, _| {
        let proxy = AccumulatorProxy::install(cx, data.acc.clone());

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

            // Panel::new(cx, "filters", |cx| {
            //     for fparam in params.filters.iter() {
            //         param_knob(cx, &fparam.cutoff);
            //         param_selector(cx, &fparam.kind);
            //     }
            // });

            Panel::new(cx, "N Analyzer", |cx| {
                proxy
                    .rolling_window::<RmsBucket>(cx, MonoConsumerKind::Averaged, 0.25)
                    .height(Units::Stretch(1.));
            });
        });
    })
}
