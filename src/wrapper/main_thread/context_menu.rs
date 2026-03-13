use clack_extensions::context_menu::{
    ContextMenuBuilder, ContextMenuTarget, PluginContextMenuImpl,
};
use clack_plugin::{plugin::PluginError, utils::ClapId};

use crate::wrapper::{ClapPlugin, main_thread::WrapperMainThread};

impl<'a, P> WrapperMainThread<'a, P>
where
    P: ClapPlugin,
{
    fn populate_param_menu(
        &self,
        _: &mut ContextMenuBuilder,
        _: ClapId,
    ) -> Result<(), PluginError> {
        // todo!()
        Ok(())
    }

    fn populate_global_menu(&self, _: &mut ContextMenuBuilder) -> Result<(), PluginError> {
        // todo!()
        Ok(())
    }
}

impl<'a, P> PluginContextMenuImpl for WrapperMainThread<'a, P>
where
    P: ClapPlugin,
{
    fn populate(
        &mut self,
        target: ContextMenuTarget,
        builder: &mut ContextMenuBuilder,
    ) -> Result<(), PluginError> {
        log::info!("POPULATE");
        match target {
            ContextMenuTarget::Global => self.populate_global_menu(builder),
            ContextMenuTarget::Param(id) => self.populate_param_menu(builder, id),
            _ => Ok(()),
        }
    }

    fn perform(&mut self, _: ContextMenuTarget, _: ClapId) -> Result<(), PluginError> {
        // todo!()
        // maybe add new things later, for now host does it
        log::info!("perform");
        Ok(())
    }
}
