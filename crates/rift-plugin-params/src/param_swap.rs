//! A host-invisible ANY `T` parameter.
//!
//! CLAP parameters are `f64`-valued, so a T cannot be exposed to the host.
//! [`SwapParam`] is therefore a [`Persistent`] value: it carries a stable
//! `ClapId` (for `on_param_change`-style notifications) and is saved under the
//! `persist` section of the plugin state.

use std::io::{Read, Write};
use std::ops::Deref;
use std::sync::Arc;

use arc_swap::ArcSwap;
use clack_plugin::plugin::PluginError;
use clack_plugin::utils::ClapId;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::Persistent;

/// A shared T parameter: a host-invisible, [`Persistent`] value.
///
/// Build manually with [`SwapParam::builder`], or let `#[derive(Params)]`
/// construct it via [`Persistent::create`].
#[derive(bon::Builder)]
pub struct SwapParam<T>
where
    T: Default + Sized + Clone + Serialize + DeserializeOwned,
{
    /// Value used until (and if) the saved state provides one.
    #[builder(default)]
    default: T,

    #[builder(skip = ArcSwap::from_pointee(default.clone()))]
    value: ArcSwap<T>,

    /// Initialized by the derive with the field name and module.
    #[builder(default)]
    name: String,

    module: Option<String>,

    #[builder(default = ClapId::new(0))]
    id: ClapId,
}

impl<T> SwapParam<T>
where
    T: Default + Sized + Clone + Serialize + DeserializeOwned,
{
    /// A snapshot of the current value.
    ///
    /// Lock-free and allocation-free, so it is safe to call from the audio
    /// thread. The returned guard keeps the value alive while it is held.
    #[inline]
    pub fn value(&self) -> SwapParamValue<T> {
        SwapParamValue(self.value.load())
    }

    /// Publishes a new value.
    ///
    /// This allocates: call it from the main/GUI thread, never from the audio
    /// thread on the hot path.
    #[inline]
    pub fn set_value(&self, value: impl Into<T>) {
        self.value.store(Arc::new(value.into()));
    }

    /// The value used until (and if) the saved state provides one.
    #[inline]
    pub fn default_value(&self) -> &T {
        &self.default
    }
}

pub struct SwapParamValue<T>(arc_swap::Guard<Arc<T>>)
where
    T: Default + Sized + Clone + Serialize + DeserializeOwned;

impl<T> Deref for SwapParamValue<T>
where
    T: Default + Sized + Clone + Serialize + DeserializeOwned,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::fmt::Debug for SwapParamValue<T>
where
    T: Default + Sized + Clone + Serialize + DeserializeOwned + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&**self, f)
    }
}

impl<T> Persistent for SwapParam<T>
where
    T: Default + Sized + Clone + Serialize + DeserializeOwned,
{
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
            .map_err(|_| PluginError::Message("Failed to serialize SwapParam"))
    }

    fn deserialize(&self, reader: &mut dyn Read) -> Result<(), PluginError> {
        let value: T = serde_json::from_reader(reader)
            .map_err(|_| PluginError::Message("Failed to deserialize SwapParam"))?;
        self.set_value(value);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use clack_plugin::utils::ClapId;
    use serde::{Deserialize, Serialize};

    use crate::Persistent;

    use super::*;

    /// A non-trivial `T`: exercises the generic path with a type that is not
    /// `String`, proving `SwapParam` is not specialised to strings.
    #[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
    struct Settings {
        size: u32,
        label: String,
        enabled: bool,
    }

    fn settings() -> Settings {
        Settings {
            size: 8,
            label: String::from("verb"),
            enabled: true,
        }
    }

    #[test]
    fn defaults_to_t_default() {
        let param: SwapParam<Settings> = SwapParam::builder().build();

        assert_eq!(&*param.value(), &Settings::default());
        assert_eq!(param.default_value(), &Settings::default());
    }

    #[test]
    fn builder_default_is_the_initial_value() {
        let default = settings();
        let param = SwapParam::builder().default(default.clone()).build();

        assert_eq!(&*param.value(), &default);
        assert_eq!(param.default_value(), &default);
    }

    #[test]
    fn set_value_publishes_and_leaves_default_untouched() {
        let default = Settings::default();
        let param = SwapParam::builder().default(default.clone()).build();

        param.set_value(settings());

        assert_eq!(&*param.value(), &settings());
        assert_eq!(param.default_value(), &default);
    }

    #[test]
    fn held_snapshot_survives_later_writes() {
        let param = SwapParam::builder().default(String::from("a")).build();

        let snapshot = param.value();
        param.set_value("b");

        // The guard keeps the old value alive even after it is replaced.
        assert_eq!(&*snapshot, "a");
        assert_eq!(&*param.value(), "b");
    }

    #[test]
    fn set_value_accepts_anything_into_t() {
        let param: SwapParam<String> = SwapParam::builder().build();

        // `&str` goes through `impl Into<String>`, matching `StringParam`.
        param.set_value("owned");
        assert_eq!(&*param.value(), "owned");
    }

    #[test]
    fn debug_prints_the_inner_value() {
        let param = SwapParam::builder().default(settings()).build();

        assert_eq!(format!("{:?}", param.value()), format!("{:?}", settings()));
    }

    #[test]
    fn persistent_identity() {
        let param = SwapParam::<Settings>::create(
            ClapId::new(11),
            String::from("reverb"),
            Some(String::from("fx")),
        );

        assert_eq!(param.id(), ClapId::new(11));
        assert_eq!(param.name(), "reverb");
        assert_eq!(param.module(), Some("fx"));
        assert_eq!(param.path(), "fx.reverb");
        // `create` has no value to restore yet, so it starts at `T::default()`.
        assert_eq!(&*param.value(), &Settings::default());
    }

    #[test]
    fn identity_without_module() {
        let param = SwapParam::<Settings>::create(ClapId::new(3), String::from("solo"), None);

        assert_eq!(param.module(), None);
        assert_eq!(param.path(), "solo");
    }

    #[test]
    fn persistent_roundtrip() {
        let param = SwapParam::<Settings>::create(ClapId::new(1), String::from("s"), None);
        param.set_value(settings());

        let mut buf = Vec::new();
        param.serialize(&mut buf).unwrap();

        // The wire format is a plain JSON encoding of the inner `T`.
        assert_eq!(
            serde_json::from_slice::<Settings>(&buf).unwrap(),
            settings()
        );

        let restored = SwapParam::<Settings>::create(ClapId::new(1), String::from("s"), None);
        restored.deserialize(&mut buf.as_slice()).unwrap();

        assert_eq!(&*restored.value(), &settings());
    }

    #[test]
    fn deserialize_invalid_data_errors_and_keeps_value() {
        let param = SwapParam::<Settings>::create(ClapId::new(1), String::from("s"), None);
        param.set_value(settings());

        let mut reader = std::io::Cursor::new(b"not json");
        assert!(param.deserialize(&mut reader).is_err());

        // A failed load must not clobber the current value.
        assert_eq!(&*param.value(), &settings());
    }

    #[test]
    fn works_with_byte_blobs() {
        let param: SwapParam<Vec<u8>> = SwapParam::builder().build();
        assert!(param.value().is_empty());

        param.set_value(vec![1u8, 2, 3]);
        assert_eq!(&*param.value(), &vec![1u8, 2, 3]);
    }

    #[test]
    fn is_send_and_sync() {
        // The value has to cross threads: written on the main/GUI thread and
        // read from the audio thread.
        fn assert_send_sync<T: Send + Sync>() {}

        assert_send_sync::<SwapParam<Settings>>();
        assert_send_sync::<SwapParamValue<Settings>>();
    }
}
