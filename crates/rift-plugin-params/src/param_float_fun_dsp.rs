use fundsp::prelude32::{An, Shared, var};

use clack_extensions::params::*;
use clack_plugin::utils::ClapId;
use fundsp::shared::Var;

use super::ptr::ParamPtr;
use super::traits::{Param, TypedParam};

use crate::Scale;

#[derive(bon::Builder)]
pub struct SharedFloatParam {
    /// The name of the param will
    /// be initialized in the derive with it's clap ID
    /// and module.
    #[builder(default)]
    name: String,

    module: Option<String>,

    #[builder(default)]
    pub(crate) default: f32,

    #[builder(skip = Shared::new(default))]
    pub(crate) value: Shared,

    #[builder(default = "")]
    pub(crate) unit: &'static str,

    #[builder(default = 0.0)]
    pub(crate) min_value: f32,

    #[builder(default = 1.0)]
    pub(crate) max_value: f32,

    #[builder(default)]
    pub(crate) scale: Scale,

    #[builder(default = ParamInfoFlags::IS_AUTOMATABLE)]
    pub(crate) flags: ParamInfoFlags,

    #[builder(default = ClapId::new(0))]
    pub(crate) id: ClapId,
}

impl TypedParam for SharedFloatParam {
    type Type = f32;

    fn value(&self) -> Self::Type {
        self.value.value()
    }

    #[inline]
    fn set_value(&self, value: Self::Type) {
        self.value.set(value.clamp(self.min_value, self.max_value));
    }
}

impl crate::traits::__private::Sealed for SharedFloatParam {}

impl Param for SharedFloatParam {
    fn name(&self) -> &str {
        &self.name
    }

    fn module(&self) -> Option<&str> {
        self.module.as_deref()
    }

    fn id(&self) -> ClapId {
        self.id
    }

    fn unit(&self) -> &str {
        self.unit
    }

    fn set_plain(&self, value: f32) {
        self.set_value(value);
    }

    fn plain(&self) -> f32 {
        self.value.value()
    }

    fn default_plain(&self) -> f32 {
        self.default
    }

    fn normalized(&self) -> f32 {
        let value = self.plain();
        self.normalize(value)
    }

    fn set_normalized(&self, normalized: f32) {
        self.set_plain(self.denormalize(normalized));
    }

    fn flags(&self) -> ParamInfoFlags {
        self.flags
    }

    fn min_value(&self) -> f32 {
        self.min_value
    }

    fn max_value(&self) -> f32 {
        self.max_value
    }

    fn normalize(&self, value: f32) -> f32 {
        self.scale.normalize(value, self.min_value, self.max_value)
    }

    fn denormalize(&self, normalized: f32) -> f32 {
        self.scale
            .denormalize(normalized, self.min_value, self.max_value)
    }

    fn as_ptr(&self) -> ParamPtr {
        ParamPtr::new(self as *const dyn Param)
    }
}

impl SharedFloatParam {
    /// Returns a fundsp [`Var`] audio node that outputs the current value of
    /// this param on every sample.
    ///
    /// The node reads the same [`Shared`] the host automates, so plugging it into
    /// a graph makes that graph follow the parameter with no manual copying. It is
    /// shorthand for `var(&self.value)`.
    ///
    /// # Examples
    ///
    /// The two filters below are equivalent. First, using the underlying fundsp
    /// [`Shared`] directly:
    ///
    /// ```
    /// use fundsp::prelude32::*;
    ///
    /// let cutoff = Shared::new(440f32);
    /// let mono_filter = || (pass() | var(&cutoff) | dc(0.5f32)) >> lowpass();
    /// ```
    ///
    /// Then, inside a `#[derive(Params)]` struct where `cutoff` is a
    /// [`SharedFloatParam`] field:
    ///
    /// ```ignore
    /// use fundsp::prelude32::*;
    ///
    /// #[derive(Params)]
    /// struct Params {
    ///     #[param(name = "Cutoff", range = 20..20000, default = 440.0)]
    ///     cutoff: SharedFloatParam,
    /// }
    ///
    /// let params = Params::default();
    /// let mono_filter = || (pass() | params.cutoff.var() | dc(0.5f32)) >> lowpass();
    /// ```
    pub fn var(&self) -> An<Var> {
        var(&self.value)
    }
}
