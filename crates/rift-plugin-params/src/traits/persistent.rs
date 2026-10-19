use std::io::{Read, Write};

use clack_plugin::prelude::PluginError;
use clack_plugin::utils::ClapId;

use crate::Tags;

/// Trait for plugin state persistence (preset save/load). This represent any param that
/// cannot be represented as a float. Host will never know about this parameter. This is all internal
/// in this plugin.
///
/// Each implementor is responsible for writing a single valid JSON value to the writer and reading it back.
pub trait Persistent {
    fn create(id: ClapId, name: String, module: Option<String>, tags: impl Into<Tags>) -> Self
    where
        Self: Sized;
    fn name(&self) -> &str;
    fn module(&self) -> Option<&str>;
    fn path(&self) -> String {
        if let Some(module) = self.module() {
            format!("{}.{}", module, self.name())
        } else {
            self.name().to_string()
        }
    }
    fn id(&self) -> ClapId;
    fn serialize(&self, writer: &mut dyn Write) -> Result<(), PluginError>;
    fn deserialize(&self, reader: &mut dyn Read) -> Result<(), PluginError>;
    fn tags(&self) -> Tags;
}
