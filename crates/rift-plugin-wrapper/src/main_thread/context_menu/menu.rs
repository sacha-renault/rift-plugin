use clack_extensions::context_menu::{
    ContextMenuBuilder, ContextMenuTarget, PluginContextMenuImpl,
};
use clack_plugin::{plugin::PluginError, utils::ClapId};

use crate::{ClapPlugin, main_thread::WrapperMainThread};

use super::menu_item::MenuItem;

/// The menu `P` declares for `target`, or `None`
fn menu_items<P: ClapPlugin>(target: ContextMenuTarget) -> Option<&'static [MenuItem]> {
    match target {
        ContextMenuTarget::Global => Some(P::GLOBAL_MENU_ITEMS),
        ContextMenuTarget::Param(_) => Some(P::PARAM_MENU_ITEMS),
        _ => None,
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
        log::debug!("POPULATE");

        let Some(items) = menu_items::<P>(target) else {
            log::error!("No context menu to populate for target {target:?}");
            return Ok(());
        };

        super::menu_item::build_context_menu(items, builder, &self.menu_context(target))
            .map_err(|err| err.into())
    }

    fn perform(&mut self, target: ContextMenuTarget, action_id: ClapId) -> Result<(), PluginError> {
        log::debug!("perform");

        let Some(items) = menu_items::<P>(target) else {
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
