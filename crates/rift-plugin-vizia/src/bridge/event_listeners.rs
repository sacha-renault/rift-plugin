use std::sync::Arc;

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

        event.map(|e: &ParamContextMenuRequest, meta| {
            host.param_context_menu(e.id, e.x, e.y, e.screen);
            meta.consume();
        });
    });

    ParamRegistry::new().build(cx);
}

/// Applies what the host sent since the last call (automation, preset loads...)
/// to the widgets. Call it on every frame, e.g. from `on_idle`.
pub fn pump(cx: &mut Context, host: &dyn GuiContext) {
    // audio thread => GUI
    let registry = ParamRegistry::from_context(cx);
    while let Some(change) = host.pop_in_gui() {
        registry.apply_host_value(change.id, change.value);
    }

    if host.take_refresh_request() {
        registry.refresh_param_values();
    }

    if let Some(info) = host.latest_transport_info() {
        BundledTransportSignals::from_context(cx).update(info);
    }
}

pub fn install_common_signals(cx: &mut Context) {
    BundledTransportSignals::new().install(cx);
}

#[cfg(test)]
mod tests {
    use rift_plugin_types::TransportInfo;
    use rift_plugin_utils::DEFAULT_SAMPLERATE;

    use super::*;

    // Any signal trying to be updated that isn't installed will trigger
    // an error here.
    #[test]
    fn updated_signals_are_installed() {
        let mut cx = Context::new();
        install_common_signals(&mut cx);
        BundledTransportSignals::from_context(&mut cx)
            .update(TransportInfo::new(DEFAULT_SAMPLERATE));
    }
}
