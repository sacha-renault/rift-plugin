//! Entry point used by plugins: turns an app closure into a [`GuiFactory`].

use std::sync::Arc;

use rift_plugin_gui::{ClapGui, GuiContext, GuiFactory, GuiSize};
use vizia::prelude::Context;

use crate::gui::ViziaGui;

/// Builds a [`ViziaGui`] from an application closure and a fixed window size.
///
/// Prefer the [`vizia_gui`] helper over constructing this type directly.
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
        let size = GuiSize {
            width: self.width,
            height: self.height,
        };
        Box::new(ViziaGui::new(Arc::new(self.app_fn), size, ctx))
    }
}

/// Creates the [`GuiFactory`] of a plugin from a vizia application closure.
///
/// ```ignore
/// fn gui(params: Arc<MyParams>, _data: Arc<()>) -> Box<dyn GuiFactory> {
///     vizia_gui(320, 220, move |cx, _| {
///         Panel::new(cx, "Oscillator", |cx| {
///             param_knob(cx, &params.gain);
///         });
///     })
/// }
/// ```
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
