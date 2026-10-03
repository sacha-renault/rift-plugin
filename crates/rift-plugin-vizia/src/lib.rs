use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use rift_plugin_gui::*;
use vizia::{
    Application, WindowHandle,
    context::EmitContext,
    prelude::{Context, WindowSize},
};

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

    fn set_parent(&mut self, window: Window) -> Result<(), PluginError> {
        self.parent = Some(window.raw_window_handle());
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

unsafe impl HasRawWindowHandle for ViziaGui {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.parent.expect("Window Handle isn't available")
    }
}

impl ViziaGui {
    fn spawn(&mut self) -> Result<(), PluginError> {
        if self.parent.is_none() {
            return Err(PluginError::Message("Parent wasn't set."));
        }

        let app_fn = self.app_fn.clone();
        let context = self.context.clone();
        let idle_ctx = self.context.clone();
        let wsize = WindowSize::new(self.size.width, self.size.height);

        let application = Application::new(move |cx| {
            app_fn(cx, context.clone());
        })
        .inner_size(wsize)
        .on_idle(move |cx| {
            // Here we have to pump param change bidirectionally
            // audio thread => GUI
            // GUI          => audio thread
            while let Some(task) = idle_ctx.pop_gui_task() {
                cx.emit(task);
            }
        });

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

pub mod prelude {
    pub use super::vizia_gui;
    pub use rift_plugin_gui::GuiFactory;
    pub use vizia::prelude::*;
}
