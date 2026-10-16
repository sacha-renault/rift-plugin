use clack_extensions::context_menu::ContextMenuTarget;
use rift_plugin_context::MenuContext;

use crate::{ClapPlugin, main_thread::WrapperMainThread};

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
}
