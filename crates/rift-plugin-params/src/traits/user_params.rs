use clack_plugin::prelude::PluginError;
use clack_plugin::utils::ClapId;

use super::Param;
use crate::{ParamPtr, Tags};

/// Implemented by the `#[derive(Params)]` structs written by the user.
///
/// It exposes every leaf parameter reachable from the struct (walking nested
/// structs and arrays) as a type-erased [`ParamPtr`], which the wrapper then
/// collects into a [`ParamCollection`](crate::ParamCollection) map. It also owns
/// the (de)serialization of both the parameter values and any extra `#[persist]`
/// state.
///
/// Only [`UserParams::all_params`] has to be implemented by the derive; the
/// parameter methods below are provided from it. The persist methods are
/// defaulted no-ops and get overridden once `#[persist]` fields exist.
pub trait UserParams {
    /// Every parameter reachable from this struct, in declaration order.
    fn all_params(&self) -> Vec<ParamPtr>;

    /// The [`ClapId`]s of every parameter whose [`tags`](Param::tags) satisfy
    /// `predicate`.
    ///
    /// [`Tags`] is a bare `u32`, so nothing stops you from tagging params with
    /// two different flag structs, say `ParamFlags` and `ParamFlags2`, with the
    /// bits laid out in a different order. It compiles, the compiler never
    /// complains, and the results are then incoherent asf: we only ever AND bits
    /// here, so we have no idea which struct a param was tagged with.
    ///
    /// Not fixing it by giving params another `<Tag>` either. There's already
    /// enough blanket impl flying in every direction in this crate. Normally you
    /// define a single tag struct and it never comes up, just extra care if you use
    /// more than 1.
    fn params_by_tags<F>(&self, predicate: F) -> impl Iterator<Item = ClapId>
    where
        F: Fn(Tags) -> bool,
    {
        self.all_params()
            .into_iter()
            .filter(move |param| predicate(param.tags()))
            .map(|param| param.id())
    }

    /// Write every reachable parameter into `out`, keyed by its stable
    /// [`ClapId`], as its plain value.
    ///
    /// [`ClapId`]: clack_plugin::utils::ClapId
    fn serialize_params(
        &self,
        out: &mut serde_json::Map<String, serde_json::Value>,
    ) -> Result<(), PluginError> {
        for param in self.all_params() {
            // Keyed by the stable `ClapId` rather than by name, so that renaming
            // a label does not invalidate previously saved state.
            out.insert(
                param.id().get().to_string(),
                serde_json::Value::from(param.plain() as f64),
            );
        }

        Ok(())
    }

    /// Read parameters from `section`, keyed by their stable [`ClapId`].
    ///
    /// Missing, unknown and non-numeric entries are ignored: this keeps state
    /// loadable across plugin updates that add or remove parameters.
    ///
    /// [`ClapId`]: clack_plugin::utils::ClapId
    fn deserialize_params(
        &self,
        section: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<(), PluginError> {
        for param in self.all_params() {
            let key = param.id().get().to_string();

            if let Some(value) = section.get(&key).and_then(serde_json::Value::as_f64) {
                param.set_plain(value as f32);
            }
        }

        Ok(())
    }

    /// Write extra, non-parameter state into `out`.
    ///
    /// Defaulted to a no-op; `#[derive(Params)]` overrides it once `#[persist]`
    /// fields are supported.
    fn serialize_persist(
        &self,
        _out: &mut serde_json::Map<String, serde_json::Value>,
    ) -> Result<(), PluginError> {
        Ok(())
    }

    /// Read extra, non-parameter state from `section`.
    ///
    /// Defaulted to a no-op; `#[derive(Params)]` overrides it once `#[persist]`
    /// fields are supported.
    fn deserialize_persist(
        &self,
        _section: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<(), PluginError> {
        Ok(())
    }
}
