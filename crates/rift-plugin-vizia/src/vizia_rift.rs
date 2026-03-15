//! Contains all types and implementations related to Gui window managementb

use clack_extensions::gui::{GuiSize, Window};
use clack_plugin::plugin::PluginError;

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use vizia::prelude::*;

use rift_plugin_shared::gui::*;

use crate::utils::gui_events::ContextMenuEvent;

pub struct ViziaGuiFactory<F> {
    app_fn: Arc<F>,
    size: (u32, u32),
}

impl<F> GuiFactory for ViziaGuiFactory<F>
where
    F: Fn(&mut Context, Arc<dyn GuiContext>) + Send + Sync + 'static,
{
    fn build(self: Box<Self>, context: Arc<dyn GuiContext>) -> Box<dyn ClapGui> {
        Box::new(ViziaGui {
            parent: None,
            handle: None,
            app_fn: self.app_fn,
            opened: Arc::new(AtomicBool::new(false)),
            size: self.size,
            context,
        })
    }
}

#[derive(Lens)]
struct ViziaData {
    ctx: Arc<dyn GuiContext>,
}

impl Model for ViziaData {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|app_event: &GuiParamEvent, meta| {
            if let GuiParamEventKind::Value(v) = app_event.kind {
                self.ctx.params().set_value(app_event.param_id, v)
            }
            self.ctx.param_event(*app_event);
            meta.consume();
        });

        event.map(|&ContextMenuEvent(param_id): &ContextMenuEvent, meta| {
            let mouse = cx.mouse();

            let x = mouse.cursor_x as i32;
            let y = mouse.cursor_y as i32;
            self.ctx.param_context_menu(param_id, x, y, 0);
            meta.consume();
        });
    }
}

pub struct ViziaGui<F> {
    /// Holds raw handle to parent window.
    parent: Option<RawWindowHandle>,
    /// Holds handle to plugin window.
    handle: Option<WindowHandle>,
    /// Know if it's opened or not
    opened: Arc<AtomicBool>,
    /// the fn that will be used for mainloop in ViziaApp
    app_fn: Arc<F>,
    /// (width, height)
    size: (u32, u32),
    /// States
    context: Arc<dyn GuiContext>,
}

unsafe impl<F> HasRawWindowHandle for ViziaGui<F> {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.parent.unwrap()
    }
}

impl<F> ViziaGui<F>
where
    F: Fn(&mut Context, Arc<dyn GuiContext>) + Send + Sync + 'static,
{
    pub fn factory(size: (u32, u32), app_fn: F) -> ViziaGuiFactory<F> {
        ViziaGuiFactory {
            app_fn: Arc::new(app_fn),
            size,
        }
    }
}

impl<F> ClapGui for ViziaGui<F>
where
    F: Fn(&mut Context, Arc<dyn GuiContext>) + Send + Sync + 'static,
{
    fn spawn(&mut self) {
        let app_fn = self.app_fn.clone();
        let context = self.context.clone();

        let application = Application::new(move |cx| {
            ViziaData {
                ctx: context.clone(),
            }
            .build(cx);
            app_fn(cx, context.clone());
        })
        .inner_size(self.size);

        self.handle = Some(application.open_parented(self));
        self.opened.store(true, Ordering::Relaxed);
        log::info!("ClapGui::spawn was called")
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
        self.spawn();
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
        Ok(())
    }

    fn set_transient(&mut self, _window: Window) -> Result<(), PluginError> {
        Ok(())
    }

    fn set_scale(&mut self, _scale: f64) -> Result<(), PluginError> {
        Err(PluginError::Message("Unsupported"))
    }
}
