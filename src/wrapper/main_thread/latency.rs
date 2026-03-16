use clack_extensions::latency::PluginLatencyImpl;

use crate::prelude::*;
use crate::wrapper::ClapPlugin;

impl<'a, P: ClapPlugin> PluginLatencyImpl for super::WrapperMainThread<'a, P> {
    fn get(&mut self) -> u32 {
        log::debug!("get (latency)");
        self.shared.states.get_latency()
    }
}
