use core::f32;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use rift_plugin::prelude::utils::conversion::linear_to_db;
use rift_plugin::prelude::*;

use rift_plugin_accumulator::prelude::*;
use rift_plugin_resolver::resolve_css_variables_from_list;
use rift_plugin_vizia::*;

use AudioConsumerDispatch as ACD;

use crate::params::PluginParams;
use crate::shared::Shared;

pub const THEMES_VAR: &[(&str, &str)] = &[
    ("background-color", "#0f0f0f"),
    ("background-secondary", "#1a1a1a"),
    ("background-tertiary", "#2d1b2e"),
    ("primary-color", "#e91e63"),
    ("secondary-color", "#9c27b0"),
    ("accent-color", "#673ab7"),
    ("accent-secondary", "#4a148c"),
    ("text-primary", "#ffffff"),
    ("text-secondary", "#cccccc"),
    ("text-accent", "#f06292"),
    ("border-color", "#2d1b2e"),
    ("border-light", "#4a148c"),
    ("hover-color", "#673ab7"),
    ("active-color", "#e91e63"),
    ("shadow-color", "#0f0f0f"),
    ("gradient-start", "#e91e63"),
    ("gradient-mid", "#9c27b0"),
    ("gradient-end", "#673ab7"),
    ("surface-color", "#1a1a1a"),
    ("surface-elevated", "#2d1b2e"),
    // Extras
    ("focus-color", "#e91e63"),
    ("disabled-color", "#555555"),
    ("primary-alpha-25", "#e91e634d"),
    ("primary-alpha-20", "#e91e6333"),
    ("primary-alpha-10", "#e91e631a"),
    ("primary-alpha-50", "#e91e6380"),
    ("secondary-alpha-25", "#9c27b040"),
    ("background-alpha-08", "#1a1a1a14"),
    ("shadow-alpha-22", "#0f0f0f38"),
];

#[derive(Lens)]
pub struct AppData {
    params: Arc<PluginParams>,
    shared: Arc<Shared>,
    _is_playing: Arc<AtomicBool>,
}

impl Model for AppData {}

impl AppData {
    fn accumulator() -> impl Lens<Target = AudioAccumulator> {
        AppData::shared.map(|s| s.post_acc.clone())
    }
}

#[derive(Lens)]
pub struct AudioConsumers {
    audio_peaks: ConsumerCell<MultiChannel<AudioPeak>>,
}

impl Model for AudioConsumers {}

pub fn create_gui(params: Arc<PluginParams>, shared: Arc<Shared>) -> Box<dyn GuiFactory> {
    ViziaGui::factory((900, 600), move |cx, ctx| {
        let raw_style = include_str!("../style.css");
        let style = resolve_css_variables_from_list(&raw_style, THEMES_VAR);
        if cx.add_stylesheet(style).is_err() {
            log::error!("Failed to load style");
        }

        let params = params.clone();
        let shared = shared.clone();
        let wave_acc = WindowBuckets::<PeakBucket>::new(48000., 5.).wraps_consumer();
        let audio_peaks =
            MultiChannel::new(2, || AudioPeak::new().lerp_factor(0.6)).wraps_consumer();
        let stft = StftConsumer::new(2 << 12, 48000.).wraps_consumer();

        AppData {
            params,
            shared,
            _is_playing: ctx.is_playing(),
        }
        .build(cx);
        AudioConsumers {
            audio_peaks: audio_peaks.clone(),
        }
        .build(cx);

        let redraw_lens = ACD::new(cx, AppData::accumulator())
            .add_consumer_averaged(wave_acc.clone())
            .add_consumer_all(audio_peaks.clone())
            .add_consumer_at_channel(stft.clone(), 0)
            .redraw_lens();

        VStack::new(cx, |cx| {
            HStack::new(cx, |cx| {
                ZStack::new(cx, |cx| {
                    PlotGrid::new(cx, GridScale::logarithmic(20., 20. * 1e3, 2., 0))
                        .color(Color::gray())
                        .opacity(0.4)
                        .orientation(Orientation::Horizontal);

                    PlotGrid::new(cx, GridScale::logarithmic(20., 20. * 1e3, 2., 8))
                        .color(Color::gray())
                        .opacity(0.2)
                        .orientation(Orientation::Horizontal);

                    PlotGrid::new(cx, GridScale::linear(0., 1.0, 3))
                        .color(Color::gray())
                        .opacity(0.4)
                        .orientation(Orientation::Vertical);

                    // PlotXY::new(cx, stft.clone())
                    //     .redraw_on(redraw_lens)
                    //     .use_cache_texture()
                    //     .color(Color::purple())
                    //     .resolution(3.)
                    //     .y_range((-60., -6.))
                    //     .fill_lign_height(-60.)
                    //     .border_width(Pixels(2.0));

                    PlotXY::new(cx, wave_acc.clone())
                        .redraw_on(redraw_lens)
                        .use_cache_texture()
                        .color(Color::purple())
                        .resolution(0.25)
                        .fill_lign_height(0.)
                        .y_range((0., 0.2))
                        .border_width(Pixels(2.0));
                })
                .width(Stretch(1.))
                .height(Stretch(1.))
                .right(Pixels(0.));
            });

            HStack::new(cx, |cx| {
                ParamKnob::new(AppData::params, |p| &p.clip)
                    .knob_modifiers(|knob| knob.width(Pixels(60.)).height(Pixels(60.)))
                    .label_text_modifier(|label| label.text_align(TextAlign::Center))
                    .build_view(cx)
                    .height(Pixels(100.))
                    .width(Pixels(100.));
                // .popup_on(dropdown, AppData::params.map(|p| p.skip.value()));

                ParamKnob::new(AppData::params, |p| &p.gain)
                    .knob_modifiers(|knob| knob.width(Pixels(60.)).height(Pixels(60.)))
                    .label_text_modifier(|label| label.text_align(TextAlign::Center))
                    .value_text_formater(|value, unit| {
                        format!("{:>6.1} {}", linear_to_db(value as f32), unit)
                    })
                    .span(15.)
                    .build_view(cx)
                    .height(Pixels(100.))
                    .width(Pixels(100.));

                ParamKnob::new(AppData::params, |p| &p.lfo_time)
                    .knob_modifiers(|knob| knob.width(Pixels(60.)).height(Pixels(60.)))
                    .label_text_modifier(|label| label.text_align(TextAlign::Center))
                    // .taper_inverse(|v| v * v * v)
                    // .taper(|v| v.cbrt())
                    // .value_text_formater(|value, unit| {
                    //     format!("{:>6.1} {}", linear_to_db(value as f32), unit)
                    // })
                    // .knob_range((-270., 90. - 1e-6))
                    .build_view(cx)
                    .height(Pixels(100.))
                    .width(Pixels(100.));

                ParamKnob::new(AppData::params, |p| &p.cutoff)
                    .knob_modifiers(|knob| knob.width(Pixels(60.)).height(Pixels(60.)))
                    .label_text_modifier(|label| label.text_align(TextAlign::Center))
                    .build_view(cx)
                    .height(Pixels(100.))
                    .width(Pixels(100.));

                // ParamPadXY::new(AppData::params, |p| &p.clip, |p| &p.gain)
                //     .build_view(cx)
                //     .height(Pixels(100.))
                //     .width(Pixels(100.));

                ParamButton::new(AppData::params, |p| &p.skip)
                    .button_modifiers(|btn| btn.height(Pixels(30.)))
                    .build_view(cx)
                    .height(Pixels(30.));
            });

            HStack::new(cx, |cx| {
                ZStack::new(cx, |cx| {
                    PlotGrid::new(cx, GridScale::linear(0., 1., 3))
                        .color(Color::gray())
                        .opacity(0.4)
                        .orientation(Orientation::Horizontal);

                    PlotGrid::new(cx, GridScale::linear(0., 1., 3))
                        .color(Color::gray())
                        .opacity(0.2)
                        .orientation(Orientation::Horizontal);

                    PlotGrid::new(cx, GridScale::linear(0., 1., 3))
                        .color(Color::gray())
                        .opacity(0.4)
                        .orientation(Orientation::Vertical);

                    PlotXY::new(cx, wave_acc.clone())
                        .redraw_on(redraw_lens)
                        .use_cache_texture()
                        .color(Color::purple())
                        .resolution(0.25)
                        .fill_lign_height(0.)
                        .y_range((0., 0.2))
                        .border_width(Pixels(2.0));

                    // let rule =
                    //     |idx: usize, ControlPoint { x, y, tension }, points: &ControlPoints| {
                    //         if idx == 0 {
                    //             ControlPoint { x: 0., y, tension }
                    //         } else if idx == points.len() - 1 {
                    //             ControlPoint { x: 1., y, tension }
                    //         } else {
                    //             let prev_x = <[ControlPoint]>::get(points, idx.wrapping_sub(1))
                    //                 .map(|p| p.x)
                    //                 .unwrap_or(0.);
                    //             let next_x = <[ControlPoint]>::get(points, idx + 1)
                    //                 .map(|p| p.x)
                    //                 .unwrap_or(1.);
                    //             ControlPoint {
                    //                 x: x.clamp(prev_x, next_x),
                    //                 y,
                    //                 tension,
                    //             }
                    //         }
                    //     };

                    // unsafe {
                    //     ControlPointsEditor::new(
                    //         cx,
                    //         AppData::params.map(|p| p.lfo.clone()).get(cx),
                    //         rule,
                    //     )
                    //     .filled()
                    // };

                    // PositionIndicator::new(
                    //     cx,
                    //     AppData::shared.map(|s| s.lfo_position.load(Ordering::Relaxed)),
                    // )
                    // .redraw_on(redraw_lens)
                    // .visibility(
                    //     AppData::is_playing.map(|is_playing| is_playing.load(Ordering::Relaxed)),
                    // );

                    // // Fn linespace is one
                    // let closure: fn(f32) -> f32 = |x| {
                    //     let x = (x - 0.5) * 2.;
                    //     if x == 0. {
                    //         0.
                    //     } else if x < 0. {
                    //         -(-x).sqrt()
                    //     } else {
                    //         x.sqrt()
                    //     }
                    // };

                    // Oscilloscope::new(cx, closure)
                    //     .color(Color::purple())
                    //     .use_cache_texture()
                    //     .resolution(0.5)
                    //     .y_range((-1., 1.))
                    //     .filled_path(false)
                    //     .fill_lign_height(0.)
                    //     .border_width(Pixels(2.0));

                    // Binding::new(
                    //     cx,
                    //     AudioConsumers::audio_peaks.map(|ap| ap.borrow().peak(0).unwrap()),
                    //     move |cx, lens| {
                    //         let value = lens.get(cx);
                    //         let filter = move |x: f32, y: f32| {
                    //             if (x - 0.5).abs() < value {
                    //                 Some((x, y))
                    //             } else {
                    //                 None
                    //             }
                    //         };

                    //         Oscilloscope::new(cx, closure)
                    //             .color(Color::purple())
                    //             .resolution(0.5)
                    //             .y_range((-1., 1.))
                    //             .fill_lign_height(0.)
                    //             .filter_transform(Box::new(filter))
                    //             .border_width(Pixels(2.0));
                    //     },
                    // );
                })
                .width(Stretch(1.));

                VStack::new(cx, |cx| {
                    PeaksViewer::new(AudioConsumers::audio_peaks.get(cx))
                        .range((-54., 6.))
                        .graduations(vec![3., 0., -3., -6., -12., -18., -30.].into())
                        .build_view(cx)
                        .redraw_on(redraw_lens);
                })
                .alignment(Alignment::TopCenter)
                .padding_left(Pixels(10.))
                .width(Pixels(50.));
            })
            .height(Pixels(275.));
        })
        .padding(Pixels(5.0))
        .width(Stretch(1.));
    })
}
