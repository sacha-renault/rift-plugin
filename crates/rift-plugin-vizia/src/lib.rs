use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

#[allow(deprecated)] // clack only exposes the deprecated `HasRawWindowHandle` for rwh 0.6
use raw_window_handle::{HandleError, HasRawWindowHandle, HasWindowHandle, RawWindowHandle};
use rift_plugin_gui::*;
use rift_plugin_params::{ClapId, Param, ParamPtr};
use vizia::{
    Application, WindowHandle,
    context::{DataContext, EmitContext},
    events::Event,
    model::Model,
    prelude::{Context, Signal, SignalUpdate, WindowSize},
};

pub mod widgets;

pub struct ViziaFactory<F>
where
    F: Fn(&mut Context, Arc<dyn GuiContext>) + Send + Sync + 'static,
{
    width: u32,
    height: u32,
    app_fn: F,
}

impl<F> GuiFactory for ViziaFactory<F>
where
    F: Fn(&mut Context, Arc<dyn GuiContext>) + Send + Sync + 'static,
{
    fn build(self: Box<Self>, ctx: Arc<dyn GuiContext>) -> Box<dyn ClapGui> {
        Box::new(ViziaGui {
            parent: None,
            handle: None,
            opened: Arc::new(AtomicBool::new(false)),
            app_fn: Arc::new(self.app_fn),
            size: GuiSize {
                width: self.width,
                height: self.height,
            },
            context: ctx,
        })
    }
}

pub struct ViziaGui {
    /// Holds raw handle to parent window.
    parent: Option<RawWindowHandle>,
    /// Holds handle to plugin window.
    handle: Option<WindowHandle>,
    /// Know if it's opened or not
    opened: Arc<AtomicBool>,
    /// the fn that will be used for mainloop in ViziaApp
    app_fn: Arc<dyn Fn(&mut Context, Arc<dyn GuiContext>) + Send + Sync + 'static>,
    /// (width, height)
    size: GuiSize,
    /// States
    context: Arc<dyn GuiContext>,
}

impl ClapGui for ViziaGui {
    fn create(&mut self, _: GuiConfiguration) -> Result<(), PluginError> {
        Ok(())
    }

    #[allow(deprecated)] // clack only exposes the deprecated `HasRawWindowHandle` for rwh 0.6
    fn set_parent(&mut self, window: Window) -> Result<(), PluginError> {
        let raw = window
            .raw_window_handle()
            .map_err(|_| PluginError::Message("Unsupported parent window API"))?;
        self.parent = Some(raw);
        Ok(())
    }

    fn show(&mut self) -> Result<(), PluginError> {
        let result = self.spawn();
        self.opened.store(result.is_ok(), Ordering::Relaxed);
        result
    }

    fn hide(&mut self) -> Result<(), PluginError> {
        self.opened.store(false, Ordering::Relaxed);
        Ok(())
    }

    fn destroy(&mut self) {}

    fn adjust_size(&mut self, _: GuiSize) -> Option<GuiSize> {
        // todo!()
        None
    }

    fn can_resize(&mut self) -> bool {
        // todo!()
        false
    }

    fn get_size(&mut self) -> Option<GuiSize> {
        Some(self.size)
    }

    fn set_scale(&mut self, _: f64) -> Result<(), PluginError> {
        Err(PluginError::Message("Not supported"))
    }

    fn set_size(&mut self, _: GuiSize) -> Result<(), PluginError> {
        Err(PluginError::Message("Not supported"))
    }

    fn set_transient(&mut self, _: Window) -> Result<(), PluginError> {
        // baseview doesn't support floating, this is no-op
        Err(PluginError::Message(
            "Baseview doesn't support floating windows ...",
        ))
    }

    fn is_opened(&self) -> bool {
        self.opened.load(Ordering::Relaxed)
    }
}

impl HasWindowHandle for ViziaGui {
    fn window_handle(&self) -> Result<raw_window_handle::WindowHandle<'_>, HandleError> {
        let raw = self.parent.ok_or(HandleError::Unavailable)?;
        // SAFETY: The handle comes from the host, which guarantees the parent window
        // outlives the plugin's GUI as required by the CLAP GUI contract.
        Ok(unsafe { raw_window_handle::WindowHandle::borrow_raw(raw) })
    }
}

impl ViziaGui {
    fn spawn(&mut self) -> Result<(), PluginError> {
        if self.parent.is_none() {
            return Err(PluginError::Message("Parent wasn't set."));
        }

        let app_fn = self.app_fn.clone();
        let context = self.context.clone();
        let pump = context.clone();
        let wsize = WindowSize::new(self.size.width, self.size.height);

        let application = Application::new(move |cx| {
            // Here we notify RT when param change
            // GUI => audio thread
            let ctx2 = context.clone();
            cx.add_global_listener(move |_, event| {
                event.map(|e: &GuiParamEvent, meta| {
                    ctx2.param_event(*e);
                    meta.consume();
                })
            });

            ParamSignals {
                signals: HashMap::new(),
            }
            .build(cx);

            app_fn(cx, context.clone());
        })
        .on_idle(move |cx| {
            // Here we have to pump param change
            // audio thread => GUI
            let signals = &cx.data::<ParamSignals>().signals;
            while let Some(task) = pump.pop_in_gui() {
                match task {
                    GuiTasks::ParamChanged { id, value } => {
                        if let Some(data) = signals.get(&id) {
                            let normalized = data.ptr.normalize(value);
                            data.signal.update(|v| *v = normalized);
                        }
                    }
                }
            }
        })
        .inner_size(wsize);

        self.handle = Some(application.open_parented(self));
        Ok(())
    }
}

pub fn vizia_gui<F>(width: u32, height: u32, app_fn: F) -> Box<dyn GuiFactory>
where
    F: Fn(&mut Context, Arc<dyn GuiContext>) + Send + Sync + 'static,
{
    Box::new(ViziaFactory {
        width,
        height,
        app_fn,
    })
}

#[derive(Clone)]
pub struct ParamData {
    ptr: ParamPtr,
    signal: Signal<f32>,
}

pub struct ParamSignals {
    pub signals: HashMap<ClapId, ParamData>,
}

pub struct NewSignal(ParamPtr, Signal<f32>);

/// TODO, this doesn't currently work, since all registeration occures in a single frame
/// reg doesn't contain the clap id yet when a second widget is trying to register.
pub fn register_param<P: Param>(cx: &mut Context, param: &P) -> Signal<f32> {
    let signal = Signal::new(param.normalized());
    cx.emit(NewSignal(param.as_ptr(), signal));
    signal
}

impl Model for ParamSignals {
    fn event(&mut self, _: &mut vizia::prelude::EventContext, event: &mut Event) {
        event.map(|&NewSignal(ptr, signal): &NewSignal, meta| {
            self.signals.insert(ptr.id(), ParamData { ptr, signal });
            meta.consume();
        });
    }
}

pub mod prelude {
    pub use super::vizia_gui;
    pub use rift_plugin_gui::GuiFactory;
    pub use vizia::prelude::*;
}
