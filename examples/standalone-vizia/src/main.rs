use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use baseview::dpi::LogicalSize;
use baseview::{Event, EventStatus, Window, WindowContext, WindowHandler, WindowOpenOptions};

use rift_plugin::prelude::*;
use rift_plugin_gui::{
    ClapGui, GuiConfiguration, GuiContext, GuiParamEvent, GuiParamEventKind, GuiTasks,
    Window as ClapWindow,
};
use rift_plugin_vizia::prelude::*;
use rift_plugin_vizia::widgets::*;

const WIDTH: u32 = 480;
const HEIGHT: u32 = 320;

#[derive(Default, DeriveEnumValues)]
pub enum WaveType {
    #[default]
    Square,
    Saw,
    Sine,
}

#[derive(Params)]
struct StandaloneParams {
    #[param(name = "Gain", range = linear(0, 2), default = 1.0)]
    gain: FloatParam,

    #[param(name = "Cutoff", range = exp(20, 20000, 25), default = 440.0, unit = "Hz")]
    cutoff: FloatParam,

    #[param(name = "WaveType")]
    wave_type: EnumParam<WaveType>,
}

/// Minimal [`GuiContext`] for a host with no audio engine.
struct StandaloneContext {
    params: Arc<dyn ParamCollection>,
    is_playing: Arc<AtomicBool>,
}

impl GuiContext for StandaloneContext {
    fn param_event(&self, event: GuiParamEvent) {
        // With no audio thread to notify, apply value edits straight to the
        // parameters so the model stays in sync with the widgets.
        if let GuiParamEventKind::Value(value) = event.kind {
            self.params.set_value(event.param_id, value);
        }
    }

    fn params(&self) -> Arc<dyn ParamCollection> {
        self.params.clone()
    }

    fn param_context_menu(&self, _param_id: ClapId, _x: i32, _y: i32, _screen: i32) {}

    fn is_playing(&self) -> Arc<AtomicBool> {
        self.is_playing.clone()
    }

    fn pop_in_gui(&self) -> Option<GuiTasks> {
        None
    }
}

fn main() {
    let options = WindowOpenOptions::new()
        .with_title("standalone-vizia")
        .with_size(LogicalSize::new(WIDTH as f64, HEIGHT as f64));

    Window::open_blocking(options, HostWindow::new);
}

struct HostWindow {
    _gui: Box<dyn ClapGui>,
    _params: Arc<StandaloneParams>,
}

impl HostWindow {
    fn new(window: WindowContext) -> Self {
        let params = Arc::new(StandaloneParams::new());
        let context = Arc::new(StandaloneContext {
            params: Arc::new(params_wrapper::ParamsWrapper::new(params.all_params())),
            is_playing: Arc::new(AtomicBool::new(false)),
        });

        let ui_params = params.clone();
        let factory = vizia_gui(WIDTH, HEIGHT, move |cx, _| {
            cx.add_stylesheet(include_str!("style.css"))
                .expect("Failed to add");

            cx.add_stylesheet("fader { background_color: red ;}")
                .expect("Failed to load");

            HStack::new(cx, |cx| {
                VStack::new(cx, |cx| {
                    Panel::new(cx, "HSlider", |cx| {
                        param_slider(cx, &ui_params.cutoff).horizontal();
                    })
                    .width(Units::Stretch(1.));

                    Panel::new(cx, "Test", |cx| {
                        param_knob(cx, &ui_params.gain);
                        param_knob(cx, &ui_params.cutoff).centered();
                        // param_selector(cx, &ui_params.wave_type).disable_scroll();
                        param_dropdown(cx, &ui_params.wave_type);
                    });

                    Panel::new(cx, "More tests", |cx| {
                        param_toggle(cx, &ui_params.cutoff);
                        param_toggle(cx, &ui_params.cutoff).status_light();
                    });

                    // Panel::new(cx, "Buttons", |cx| {
                    //     Button::icon(cx, "Text".to_string(), icons::ICON_SHARE_OFF.to_string())
                    //         .large();
                    //     Button::icon(cx, "Text".to_string(), icons::ICON_SHARE_OFF.to_string());
                    //     Button::icon(cx, "Text".to_string(), icons::ICON_SHARE_OFF.to_string())
                    //         .small();
                    //     Button::label(cx, "No Icon".to_string());
                    // });
                })
                .width(Units::Stretch(8.));
                // .width(Units::Stretch(0.1));

                HStack::new(cx, |cx| {
                    param_slider(cx, &ui_params.cutoff)
                        .thumb_scaling(1.)
                        .line_scaling(3.);
                    // Meter::new(cx, Signal::new(0.8));
                    // Meter::new(cx, Signal::new(1.2));
                })
                .height(Units::Stretch(1.))
                .width(Units::Stretch(2.))
                .alignment(Alignment::Center);
            })
            .width(Units::Stretch(1.))
            .height(Units::Stretch(1.))
            .padding(Units::Pixels(10.));
        });

        let mut gui = factory.build(context);

        // Drive the standard lifecycle the host wrapper would call.
        gui.create(GuiConfiguration {
            api_type: clack_extensions::gui::GuiApiType::default_for_current_platform()
                .expect("no default GUI API on this platform"),
            is_floating: false,
        })
        .expect("create GUI");

        // Hand the baseview window to the GUI as its parent. `ViziaGui` reads
        // the raw handle back out of it and opens its child window on top.
        let clap_window = ClapWindow::from_window(&window)
            .expect("baseview exposes a supported raw window handle");
        gui.set_parent(clap_window).expect("set parent");
        gui.show().expect("show GUI");

        Self {
            _gui: gui,
            _params: params,
        }
    }
}

impl WindowHandler for HostWindow {
    fn on_frame(&self) {}

    fn resized(&self, _new_size: baseview::WindowSize) {}

    fn on_event(&self, _event: Event) -> EventStatus {
        EventStatus::Ignored
    }
}
