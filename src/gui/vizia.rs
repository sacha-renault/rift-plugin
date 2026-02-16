//! Contains all types and implementations related to Gui window managementb

use baseview::WindowHandle;
use clack_extensions::gui::{GuiSize, Window};
use clack_plugin::plugin::PluginError;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use vizia::prelude::*;

use crate::gui::ClapGui;

pub struct ViziaGui<F> {
    /// Holds raw handle to parent window.
    parent: Option<RawWindowHandle>,
    /// Holds handle to plugin window.
    handle: Option<WindowHandle>,
    /// Know if it's opened or not
    opened: Arc<AtomicBool>,
    /// the fn that will be used for mainloop in ViziaApp
    app_fn: Arc<F>,
    /// Size
    size: (u32, u32),
}

unsafe impl<F> HasRawWindowHandle for ViziaGui<F> {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.parent.unwrap()
    }
}

impl<F> ViziaGui<F>
where
    F: Fn(&mut Context) + Send + Sync + 'static,
{
    pub fn new(size: (u32, u32), app_fn: F) -> Self {
        Self {
            parent: None,
            handle: None,
            opened: Arc::new(AtomicBool::new(false)),
            app_fn: Arc::new(app_fn),
            size,
        }
    }

    fn handle(&self) -> &WindowHandle {
        // this should be set anyway
        self.handle.as_ref().expect("No window handle")
    }
}

impl<F> ClapGui for ViziaGui<F>
where
    F: Fn(&mut Context) + Send + Sync + 'static,
{
    fn spawn(&mut self) {
        let app_fn = self.app_fn.clone();
        let application = vizia_baseview::Application::new(move |cx| app_fn(cx))
            .inner_size(self.size)
            .on_idle(|cx| {});

        self.handle = Some(application.open_parented(self));
        self.opened.store(true, Ordering::Release);
        log::warn!("spawn was called")
    }

    fn set_size(&mut self, _size: GuiSize) -> Result<(), PluginError> {
        Err(PluginError::Message("Not Supported"))
    }

    fn adjust_size(&mut self, _size: GuiSize) -> Option<GuiSize> {
        None
    }

    fn can_resize(&mut self) -> bool {
        false
    }

    fn show(&mut self) -> Result<(), PluginError> {
        self.opened.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn hide(&mut self) -> Result<(), PluginError> {
        self.opened.store(false, Ordering::SeqCst);
        Ok(())
    }

    fn get_size(&mut self) -> Option<GuiSize> {
        let (width, height) = self.size;
        Some(GuiSize { width, height })
    }

    fn set_parent(&mut self, window: Window) -> Result<(), PluginError> {
        self.parent = Some(window.raw_window_handle());
        self.spawn();
        Ok(())
    }

    fn set_transient(&mut self, _window: Window) -> Result<(), PluginError> {
        Ok(())
    }

    fn set_scale(&mut self, _scale: f64) -> Result<(), PluginError> {
        Err(PluginError::Message("Unsupported"))
    }
}
