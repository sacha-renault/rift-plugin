//! The CLAP wrapper: the [`ClapPlugin`] trait implemented by plugins, plus the
//! glue that turns it into a CLAP plugin.

pub use clack_plugin::prelude::*;

mod clap_plugin;
mod context;

pub mod factory;
pub mod main_thread;
pub mod processor;
pub mod shared;

pub use clap_plugin::ClapPlugin;
pub use context::{InitContext, MenuContext, ProcessContext};
pub use factory::PluginWrapper;

pub use crate::shared::FromParams;
pub use main_thread::context_menu::menu_item::{ActionFn, IsCheckedFn, IsEnabledFn, MenuItem};

// Opt-in, debug-only global allocator used by `assert_no_alloc` to catch
// allocations on the realtime thread. Only installed when the feature is on.
#[cfg(all(feature = "assert-no-alloc", debug_assertions))]
#[global_allocator]
static A: assert_no_alloc::AllocDisabler = assert_no_alloc::AllocDisabler;
