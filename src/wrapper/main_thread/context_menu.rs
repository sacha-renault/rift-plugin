use clack_extensions::context_menu::{
    ContextMenuBuilder, ContextMenuTarget, PluginContextMenuImpl,
};
use clack_plugin::{plugin::PluginError, utils::ClapId};

use crate::wrapper::{ClapPlugin, main_thread::WrapperMainThread};

impl<'a, P> PluginContextMenuImpl for WrapperMainThread<'a, P>
where
    P: ClapPlugin,
{
    fn populate(
        &mut self,
        _: ContextMenuTarget,
        _: &mut ContextMenuBuilder,
    ) -> Result<(), PluginError> {
        // todo!()
        // maybe add new things later, for now host does it
        log::info!("populate");
        Ok(())
    }

    fn perform(&mut self, _: ContextMenuTarget, _: ClapId) -> Result<(), PluginError> {
        // todo!()
        // maybe add new things later, for now host does it
        log::info!("perform");
        Ok(())
    }
}
