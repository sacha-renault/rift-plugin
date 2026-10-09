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
use rift_plugin_vizia::widgets::PopupSelector;
use rift_plugin_vizia::widgets::*;

const WIDTH: u32 = 1000;
const HEIGHT: u32 = 680;

#[derive(Default, DeriveEnumValues)]
pub enum WaveType {
    #[default]
    Square,
    Saw,
    Sine,
}

#[derive(Default, DeriveEnumValues)]
pub enum FilterKind {
    #[default]
    LowPass,
    HighPass,
    BandPass,
}

#[derive(Params)]
struct StandaloneParams {
    #[param(name = "Gain", range = linear(0, 2), default = 1.0)]
    gain: FloatParam,

    #[param(name = "Cutoff", range = exp(20, 20000, 25), default = 440.0, unit = "Hz")]
    cutoff: FloatParam,

    #[param(name = "Resonance", range = linear(0, 1), default = 0.3)]
    resonance: FloatParam,

    #[param(name = "Mix", range = linear(0, 1), default = 0.5)]
    mix: FloatParam,

    #[param(name = "Drive", range = linear(0, 1), default = 0.7)]
    drive: FloatParam,

    #[param(name = "Wave")]
    wave_type: EnumParam<WaveType>,

    #[param(name = "Filter")]
    filter_kind: EnumParam<FilterKind>,
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
        .with_title("rift — widget gallery")
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
                .expect("Failed to add gallery stylesheet");

            build_gallery(cx, &ui_params);
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

/// Lays out a catalogue of every widget the theme ships.
fn build_gallery(cx: &mut Context, params: &Arc<StandaloneParams>) {
    // The theme switcher is plain UI state, not a parameter: a dropdown edits
    // a local signal and we read it to toggle a palette class on the gallery.
    let theme = Signal::new(0.0);
    let is_light = theme.map(|v| *v >= 0.25 && *v < 0.75);
    let is_ocean = theme.map(|v| *v >= 0.75);

    VStack::new(cx, |cx| {
        HStack::new(cx, |cx| {
            Label::new(cx, "Rift — Widget Gallery".to_string())
                .class("gallery-title")
                .width(Units::Stretch(1.));

            Label::new(cx, "Theme".to_string()).class("gallery-hint");
            PopupSelector::new(cx, ["Dark", "Light", "Ocean"], theme)
                .on_change(move |_, value| theme.set(value));
        })
        .class("gallery-bar")
        .width(Units::Stretch(1.));

        VStack::new(cx, |cx| {
            HStack::new(cx, |cx| {
                Panel::new(cx, "Dials", |cx| {
                    param_knob(cx, &params.drive).small();
                    param_knob(cx, &params.gain).centered();
                    param_knob(cx, &params.resonance).large();
                })
                .class("knob-panel");

                Panel::new(cx, "Sliders", |cx| {
                    VStack::new(cx, |cx| {
                        HStack::new(cx, |cx| {
                            param_slider(cx, &params.cutoff)
                                .width(Units::Pixels(56.))
                                .line_scaling(2.4);
                            param_slider(cx, &params.mix).width(Units::Pixels(56.));
                            param_slider(cx, &params.drive)
                                .width(Units::Pixels(56.))
                                .thumb_scaling(0.7);
                        })
                        .gap(Units::Stretch(1.))
                        .height(Units::Pixels(150.));

                        param_slider(cx, &params.resonance)
                            .horizontal()
                            .min_height(Units::Pixels(32.));
                    })
                    .gap(Units::Pixels(12.));
                });

                Panel::new(cx, "Toggles", |cx| {
                    VStack::new(cx, |cx| {
                        param_toggle(cx, &params.mix);
                        param_toggle(cx, &params.gain).status_light();
                        param_toggle(cx, &params.drive);
                    })
                    .alignment(Alignment::Left);
                });
            })
            .gap(Units::Pixels(10.))
            .height(Units::Stretch(1.));

            HStack::new(cx, |cx| {
                Panel::new(cx, "Selectors", |cx| {
                    VStack::new(cx, |cx| {
                        param_selector(cx, &params.wave_type);
                        param_selector(cx, &params.wave_type).arrow_select();
                        param_selector(cx, &params.filter_kind).vertical();
                    })
                    .alignment(Alignment::Left);
                });

                Panel::new(cx, "Dropdowns", |cx| {
                    VStack::new(cx, |cx| {
                        param_dropdown(cx, &params.wave_type);
                        param_dropdown(cx, &params.filter_kind);
                    })
                    .alignment(Alignment::Left);
                });

                Panel::new(cx, "Buttons", |cx| {
                    VStack::new(cx, |cx| {
                        Button::label(cx, "Undo".to_string());
                        Button::icon(cx, "Save".to_string(), icons::ICON_SHARE.to_string());
                        Button::icon(cx, "Load".to_string(), icons::ICON_DOWNLOAD.to_string())
                            .large();
                        Button::icon(cx, "Like".to_string(), icons::ICON_HEART.to_string())
                            .class("accent");
                        Button::icon(cx, "Power".to_string(), icons::ICON_POWER.to_string())
                            .reversed()
                            .small();
                    })
                    .alignment(Alignment::Left)
                    .gap(Units::Pixels(8.));
                });
            })
            .gap(Units::Pixels(12.))
            .height(Units::Stretch(1.));
        })
        .height(Units::Stretch(1.))
        .gap(Units::Pixels(12.));
    })
    .class("gallery")
    .toggle_class("gallery-light", is_light)
    .toggle_class("gallery-ocean", is_ocean)
    .gap(Units::Pixels(12.))
    .padding(Units::Pixels(16.))
    .width(Units::Stretch(1.))
    .height(Units::Stretch(1.));
}

impl WindowHandler for HostWindow {
    fn on_frame(&self) {}

    fn resized(&self, _new_size: baseview::WindowSize) {}

    fn on_event(&self, _event: Event) -> EventStatus {
        EventStatus::Ignored
    }
}
