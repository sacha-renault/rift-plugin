use clack_extensions::context_menu::{
    ContextMenuBuilder, ContextMenuTarget, PluginContextMenuImpl,
};
use clack_plugin::{plugin::PluginError, utils::ClapId};

use crate::{ClapPlugin, main_thread::WrapperMainThread};

impl<'a, P> PluginContextMenuImpl for WrapperMainThread<'a, P>
where
    P: ClapPlugin,
{
    fn populate(
        &mut self,
        target: ContextMenuTarget,
        builder: &mut ContextMenuBuilder,
    ) -> Result<(), PluginError> {
        log::debug!("POPULATE");

        let Some(items) = self.menu_items(target) else {
            log::error!("No context menu to populate for target {target:?}");
            return Ok(());
        };

        super::menu_item::build_context_menu(items, builder, &self.menu_context(target))
            .map_err(|err| err.into())
    }

    fn perform(&mut self, target: ContextMenuTarget, action_id: ClapId) -> Result<(), PluginError> {
        log::debug!("perform");

        let Some(items) = self.menu_items(target) else {
            log::error!("No context menu to perform on for target {target:?}");
            return Ok(());
        };

        let Some(action) = super::menu_item::resolve_action(items, action_id) else {
            return Err(PluginError::Message(
                "Didn't find action to perform on context menu",
            ));
        };

        action(&mut self.menu_context(target));
        Ok(())
    }
}
