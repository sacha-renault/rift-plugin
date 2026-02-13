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
        builder.register::<PluginAudioPorts>();
        // builder
        //     .register::<PluginAudioPorts>()
        //     .register::<PluginParams>()
        //     .register::<PluginState>() // todo!()
        //     .register::<PluginGui>(); // todo!()
    }
}

impl<P: ClapPlugin> DefaultPluginFactory for Wrapper<P> {
    fn get_descriptor() -> PluginDescriptor {
        P::info()
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
