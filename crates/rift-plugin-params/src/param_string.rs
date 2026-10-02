//! A host-invisible string parameter.
//!
//! CLAP parameters are `f64`-valued, so a string cannot be exposed to the host.
//! [`StringParam`] is therefore a [`Persistent`] value: it carries a stable
//! `ClapId` (for `on_param_change`-style notifications) and is saved under the
//! `persist` section of the plugin state.

use std::io::{Read, Write};
use std::ops::Deref;
use std::sync::Arc;

use arc_swap::ArcSwap;
use clack_plugin::plugin::PluginError;
use clack_plugin::utils::ClapId;

use crate::Persistent;

/// A shared string parameter: a host-invisible, [`Persistent`] value.
///
/// Build manually with [`StringParam::builder`], or let `#[derive(Params)]`
/// construct it via [`Persistent::create`].
#[derive(bon::Builder)]
pub struct StringParam {
    /// Value used until (and if) the saved state provides one.
    #[builder(default)]
    default: String,

    #[builder(skip = ArcSwap::from_pointee(default.clone()))]
    value: ArcSwap<String>,

    /// Initialized by the derive with the field name and module.
    #[builder(default)]
    name: String,

    module: Option<String>,

    #[builder(default = ClapId::new(0))]
    id: ClapId,
}

impl StringParam {
    /// A snapshot of the current value.
    ///
    /// Lock-free and allocation-free, so it is safe to call from the audio
    /// thread. The returned guard keeps the value alive while it is held.
    #[inline]
    pub fn value(&self) -> StringParamValue {
        StringParamValue(self.value.load())
    }

    /// Publishes a new value.
    ///
    /// This allocates: call it from the main/GUI thread, never from the audio
    /// thread.
    #[inline]
    pub fn set_value(&self, value: impl Into<String>) {
        self.value.store(Arc::new(value.into()));
    }

    /// The value used until (and if) the saved state provides one.
    #[inline]
    pub fn default_value(&self) -> &str {
        &self.default
    }
}

/// A lock-free, allocation-free snapshot of a [`StringParam`]'s value.
///
/// Dereferences to `str`.
pub struct StringParamValue(arc_swap::Guard<Arc<String>>);

impl Deref for StringParamValue {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.0.as_str()
    }
}

impl std::fmt::Debug for StringParamValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&**self, f)
    }
}

impl Persistent for StringParam {
    fn create(id: ClapId, name: String, module: Option<String>) -> Self {
        Self::builder()
            .id(id)
            .name(name)
            .maybe_module(module)
            .build()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn module(&self) -> Option<&str> {
        self.module.as_deref()
    }

    fn id(&self) -> ClapId {
        self.id
    }

    fn serialize(&self, writer: &mut dyn Write) -> Result<(), PluginError> {
        let value = self.value();
        serde_json::to_writer(writer, &*value)
            .map_err(|_| PluginError::Message("Failed to serialize StringParam"))
    }

    fn deserialize(&self, reader: &mut dyn Read) -> Result<(), PluginError> {
        let value: String = serde_json::from_reader(reader)
            .map_err(|_| PluginError::Message("Failed to deserialize StringParam"))?;
        self.set_value(value);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_and_set_value() {
        let param = StringParam::builder()
            .default(String::from("hello"))
            .build();

        assert_eq!(&*param.value(), "hello");
        assert_eq!(param.default_value(), "hello");

        param.set_value("world");
        assert_eq!(&*param.value(), "world");
        // Setting the value does not change the default.
        assert_eq!(param.default_value(), "hello");
    }

    #[test]
    fn default_is_empty_without_one() {
        let param = StringParam::builder().build();
        assert_eq!(&*param.value(), "");
    }

    #[test]
    fn persistent_roundtrip() {
        let param = StringParam::create(
            ClapId::new(7),
            String::from("pattern"),
            Some(String::from("seq")),
        );
        param.set_value("x..x..x.");

        let mut buf = Vec::new();
        param.serialize(&mut buf).unwrap();

        let restored = StringParam::create(ClapId::new(7), String::from("pattern"), None);
        restored.deserialize(&mut buf.as_slice()).unwrap();

        assert_eq!(&*restored.value(), "x..x..x.");
    }

    #[test]
    fn identity() {
        let param = StringParam::create(
            ClapId::new(42),
            String::from("sample"),
            Some(String::from("osc")),
        );

        assert_eq!(param.id(), ClapId::new(42));
        assert_eq!(param.name(), "sample");
        assert_eq!(param.module(), Some("osc"));
        assert_eq!(param.path(), "osc.sample");
    }

    #[test]
    fn deserialize_invalid_data() {
        let param = StringParam::builder().build();
        let mut reader = std::io::Cursor::new(b"not a string");
        assert!(param.deserialize(&mut reader).is_err());
    }
}
