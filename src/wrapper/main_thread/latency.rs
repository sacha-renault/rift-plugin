use clack_extensions::latency::PluginLatencyImpl;

use crate::prelude::*;
use crate::wrapper::ClapPlugin;

impl<'a, P: ClapPlugin> PluginLatencyImpl for super::WrapperMainThread<'a, P> {
    fn get(&mut self) -> u32 {
        self.shared.host_messages.get_latency()
    }
}
