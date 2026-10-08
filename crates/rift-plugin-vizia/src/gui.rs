//! The [`ClapGui`] implementation backed by a parented vizia/baseview window.

use std::sync::Arc;

#[allow(deprecated)] // clack only exposes the deprecated `HasRawWindowHandle` for rwh 0.6
use raw_window_handle::{HandleError, HasRawWindowHandle, HasWindowHandle, RawWindowHandle};
use rift_plugin_gui::*;
use rift_plugin_params::Param;
use vizia::{Application, WindowHandle, prelude::*};

use crate::data::ParamSignals;

/// Closure that builds the view tree of a plugin.
pub(crate) type AppFn = dyn Fn(&mut Context, Arc<dyn GuiContext>) + Send + Sync + 'static;

/// The GUI of a plugin: a vizia window embedded in the host's parent window.
pub struct ViziaGui {
    /// Holds raw handle to parent window.
    parent: Option<RawWindowHandle>,
    /// Holds handle to plugin window.
    handle: Option<WindowHandle>,
    /// The fn that will be used for mainloop in ViziaApp
    app_fn: Arc<AppFn>,
    /// (width, height)
    size: GuiSize,
    /// States
    context: Arc<dyn GuiContext>,
}

impl ViziaGui {
    pub(crate) fn new(app_fn: Arc<AppFn>, size: GuiSize, context: Arc<dyn GuiContext>) -> Self {
        Self {
            parent: None,
            handle: None,
            app_fn,
            size,
            context,
        }
    }

    fn spawn(&mut self) -> Result<(), PluginError> {
        if self.parent.is_none() {
            return Err(PluginError::Message("Parent wasn't set."));
        }

        let app_fn = self.app_fn.clone();
        let context = self.context.clone();
        let host = context.clone();
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
        self.spawn()
    }

    fn hide(&mut self) -> Result<(), PluginError> {
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

    fn param_sync(&self) -> ParamSync {
        ParamSync::Push
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
