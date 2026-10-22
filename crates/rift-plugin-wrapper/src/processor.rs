mod events;
mod lifecycle;
mod params;

use clack_plugin::prelude::*;
use rift_plugin_types::TransportInfo;

use crate::{ClapPlugin, shared::WrapperShared};

pub struct WrapperProcessor<'a, P: ClapPlugin> {
    shared: WrapperShared<P>,
    plugin: P,
    host: HostAudioProcessorHandle<'a>,
    samplerate: f64,
    transport_info: TransportInfo,

    /// Whether the audio thread pushes param changes into the GUI queue.
    /// True for `ParamSync::Push` (retained GUIs); false for `Pull`, where the
    /// GUI re-reads the values itself.
    notify_gui: bool,
}
