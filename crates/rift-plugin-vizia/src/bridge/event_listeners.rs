use std::sync::Arc;

use rift_plugin_gui::GuiTasks;

use crate::{bridge::ParamRegistry, dev_prelude::*};

/// Connects the view tree to the host.
pub fn install_event_listener(cx: &mut Context, host: Arc<dyn GuiContext>) {
    // GUI => audio thread
    // Widgets emit these events to the root; forward them to the host.
    cx.add_global_listener(move |_, event| {
        event.map(|e: &GuiParamEvent, meta| {
            host.param_event(*e);
            meta.consume();
        });
    });

    ParamRegistry::new().build(cx);
}

/// Applies what the host sent since the last call (automation, preset loads...)
/// to the widgets. Call it on every frame, e.g. from `on_idle`.
pub fn pump(cx: &Context, host: &dyn GuiContext) {
    // audio thread => GUI
    let registry = ParamRegistry::from_context(cx);
    while let Some(task) = host.pop_in_gui() {
        match task {
            GuiTasks::ParamChanged { id, value } => registry.apply_host_value(id, value),
        }
    }
}
