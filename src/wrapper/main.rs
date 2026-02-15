use std::marker::PhantomData;

use clack_extensions::{
    audio_ports::PluginAudioPorts, gui::PluginGui, params::PluginParams, state::PluginState,
};
use clack_plugin::prelude::*;

use crate::wrapper::{
    ClapPlugin, main_thread::WrapperMainThread, processor::WrapperProcessor, shared::WrapperShared,
};

pub struct Wrapper<P: ClapPlugin>(PhantomData<P>);

impl<P: ClapPlugin> Plugin for Wrapper<P> {
    type AudioProcessor<'a> = WrapperProcessor<'a, P>;
    type Shared<'a> = WrapperShared<P>;
    type MainThread<'a> = WrapperMainThread<'a, P>;

    fn declare_extensions(
        builder: &mut PluginExtensions<Self>,
        _shared: Option<&Self::Shared<'_>>,
    ) {
        builder
            .register::<PluginAudioPorts>()
            .register::<PluginAudioPorts>()
            .register::<PluginState>()
            .register::<PluginGui>();
    }
}

impl<P: ClapPlugin> DefaultPluginFactory for Wrapper<P> {
    fn get_descriptor() -> PluginDescriptor {
        // Add some compile time panics!
        const {
            if P::ID.is_empty() {
                panic!("Plugin ID must not be blank!");
            }
            if P::NAME.is_empty() {
                panic!("Plugin name must not be blank!");
            }
            if P::FEATURES.is_empty() {
                panic!("Plugin features must not be empty!");
            }
        };

        PluginDescriptor::new(P::ID, P::NAME)
            .with_description(P::DESCRIPTION)
            .with_url(P::URL)
            .with_features(P::FEATURES.to_vec())
            .with_version(P::VERSION)
            .with_vendor(P::VENDOR)
            .with_support_url(P::SUPPORT_URL)
            .with_manual_url(P::MANUAL_URL)
    }

    fn new_shared(_host: HostSharedHandle<'_>) -> Result<Self::Shared<'_>, PluginError> {
        Ok(WrapperShared::default())
    }

    fn new_main_thread<'a>(
        host: HostMainThreadHandle<'a>,
        shared: &'a Self::Shared<'a>,
    ) -> Result<Self::MainThread<'a>, PluginError> {
        Ok(WrapperMainThread {
            shared: shared.clone(),
            gui: None, // todo!()
            host,
        })
    }
}
