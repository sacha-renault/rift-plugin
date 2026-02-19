use clack_extensions::state::PluginStateImpl;
use clack_plugin::prelude::*;

use crate::prelude::*;
use crate::wrapper::ClapPlugin;

impl<'a, P: ClapPlugin> PluginStateImpl for super::WrapperMainThread<'a, P> {
    fn load(&mut self, _input: &mut clack_plugin::stream::InputStream) -> Result<(), PluginError> {
        Err(PluginError::Message("()")) // todo!()
    }

    fn save(
        &mut self,
        _output: &mut clack_plugin::stream::OutputStream,
    ) -> Result<(), PluginError> {
        Err(PluginError::Message("()")) // todo!()
    }
}
