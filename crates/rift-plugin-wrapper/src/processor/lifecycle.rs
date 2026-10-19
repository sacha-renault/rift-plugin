use clack_extensions::params::*;
use clack_plugin::events::event_types::TransportFlags;
use clack_plugin::prelude::*;

use rift_plugin_buffers::Buffers;
use rift_plugin_context::{InitContext, ProcessContext};
use rift_plugin_types::transport::BlockIndex;

use crate::ClapPlugin;
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
        let plugin = P::create(shared.params.as_ref(), audio_config, init_context);
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
            samplerate: audio_config.sample_rate,
            block_index: BlockIndex(-1),
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
            let buffers = Buffers::new(audio, P::MAIN_AUDIO_PORTS);

            if let Some(flags) = process.transport.map(|tr| tr.flags) {
                self.shared
                    .states
                    .set_is_playing(flags.contains(TransportFlags::IS_PLAYING));
            }

            let context = ProcessContext::builder()
                .host(&self.host)
                .states(self.shared.states.clone())
                .process(process)
                .samplerate(self.samplerate)
                .outputs_events(events.output)
                .block_index(self.block_index.increment())
                .host_params(self.shared.host_params.clone())
                .build();

            self.plugin.process(
                buffers,
                context,
                events.input,
                self.shared.params.as_ref(),
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
