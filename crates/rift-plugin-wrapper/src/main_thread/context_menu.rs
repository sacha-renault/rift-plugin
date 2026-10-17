use clack_extensions::context_menu::{ContextMenuTarget, HostContextMenu};
use rift_plugin_context::{MenuContext, ParamContextMenu};

use crate::{ClapPlugin, MenuItem, main_thread::WrapperMainThread};

pub mod menu;
pub mod menu_item;

impl<'a, P> WrapperMainThread<'a, P>
where
    P: ClapPlugin,
{
    fn menu_context(&self, target: ContextMenuTarget) -> MenuContext {
        let param = match target {
            ContextMenuTarget::Param(id) => Some(id),
            _ => None,
        };
        MenuContext::builder().maybe_param(param).build()
    }

    /// The menu `P` declares for `target`, or `None`
    fn menu_items(&self, target: ContextMenuTarget) -> Option<&'static [MenuItem]> {
        match target {
            ContextMenuTarget::Global => Some(P::GLOBAL_MENU_ITEMS),
            ContextMenuTarget::Param(_) => Some(P::PARAM_MENU_ITEMS),
            _ => None,
        }
    }

    pub(crate) fn open_param_menu(&mut self, param_ctx_menu: ParamContextMenu) {
        if let Some(ext) = self.host.get_extension::<HostContextMenu>() {
            let ParamContextMenu {
                param_id,
                x,
                y,
                screen,
            } = param_ctx_menu;

            if !ext.can_popup(&mut self.host) {
                log::error!("Couldn't open popup");
                return;
            }

            let target = if let Some(param_id) = param_id {
                ContextMenuTarget::Param(param_id)
            } else {
                ContextMenuTarget::Global
            };

            if let Err(err) = ext.popup(&mut self.host, target, screen, x, y) {
                log::error!("Couldn't open popup {err}")
            }
        } else {
            log::error!("Error when requesting popup open")
        }
    }
}
