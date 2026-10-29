use clack_extensions::params::*;
use clack_plugin::prelude::*;

use rift_plugin_buffers::Buffers;
use rift_plugin_types::TransportInfo;

use crate::ClapPlugin;
use crate::context::{InitContext, ProcessContext};
use crate::main_thread::WrapperMainThread;
use crate::shared::WrapperShared;

use super::WrapperProcessor;

/// Runs the audio callback under `assert_no_alloc` when the debug-only
/// `assert-no-alloc` feature is enabled, and calls it directly otherwise.
#[cfg(all(feature = "assert-no-alloc", debug_assertions))]
#[inline]
fn audio_callback<R>(f: impl FnOnce() -> R) -> R {
    assert_no_alloc::assert_no_alloc(f)
}

#[cfg(not(all(feature = "assert-no-alloc", debug_assertions)))]
#[inline]
fn audio_callback<R>(f: impl FnOnce() -> R) -> R {
    f()
}

impl<'a, P: ClapPlugin> PluginAudioProcessor<'a, WrapperShared<P>, WrapperMainThread<'a, P>>
    for WrapperProcessor<'a, P>
{
    fn activate(
        host: HostAudioProcessorHandle<'a>,
        main_thread: &mut WrapperMainThread<P>,
        shared: &'a WrapperShared<P>,
        audio_config: PluginAudioConfiguration,
    ) -> Result<Self, PluginError> {
        // Create the plugin instance & activate right away
        let init_context = InitContext::new(&main_thread.host, shared.states.clone());
        let plugin = P::create(shared.user_params.as_ref(), audio_config, init_context);
        shared.states.set_samplerate(audio_config.sample_rate);

        // After the plugin is created, we set latency
        main_thread.notify_latency_changed();

        // Only push param changes to GUIs that ask for it; pull-mode GUIs read
        // the values themselves.
        let notify_gui = matches!(
            main_thread.gui.param_sync(),
            rift_plugin_gui::ParamSync::Push
        );

        // Allocate a scratch buffer ONCE
        Ok(Self {
            shared: shared.clone(),
            plugin,
            host,
            transport_info: TransportInfo::new(audio_config.sample_rate),
            notify_gui,
        })
    }

    fn process(
        &mut self,
        process: Process,
        audio: Audio,
        events: Events,
    ) -> Result<ProcessStatus, PluginError> {
        audio_callback(|| {
            self.flush(events.input, events.output);
            let frame_count = audio.frames_count() as usize;
            let buffers = Buffers::new(audio, P::MAIN_AUDIO_PORTS);

            // update transport
            self.transport_info.update(process.transport, frame_count);
            self.shared.states.push_transport_info(self.transport_info);

            let context = ProcessContext {
                host: &self.host,
                states: self.shared.states.clone(),
                transport_info: &self.transport_info,
                output_events: events.output,
                user_params: self.shared.user_params.clone(),
                shared: self.shared.data.clone(),
                host_params: self.shared.host_params.clone(),
                notify_gui: self.notify_gui,
            };

            self.plugin.process(
                buffers,
                context,
                events.input,
                self.shared.user_params.as_ref(),
                self.shared.data.as_ref(),
            )
        })
    }

    fn deactivate(self, _: &mut WrapperMainThread<'a, P>) {
        self.plugin.deactivate();
    }

    fn reset(&mut self) {
        self.plugin.reset();
    }

    fn start_processing(&mut self) -> Result<(), PluginError> {
        self.plugin.start_processing()
    }

    fn stop_processing(&mut self) {
        self.plugin.stop_processing();
    }
}
