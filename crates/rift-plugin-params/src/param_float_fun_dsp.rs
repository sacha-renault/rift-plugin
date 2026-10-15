use fundsp::prelude32::{An, Shared, var};

use fundsp::shared::Var;

use crate::atomic_param::{Atomic, AtomicParam};
use crate::param_float::_Float;

impl Atomic<f32> for Shared {
    fn new(v: f32) -> Self {
        Self::new(v)
    }

    fn get(&self) -> f32 {
        self.value()
    }

    fn set(&self, v: f32) {
        self.set_value(v);
    }
}

pub type SharedFloatParam = AtomicParam<_Float<Shared>>;

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
    ///     #[param(name = "Cutoff", range = linear(20, 20000), default = 440.0)]
    ///     cutoff: SharedFloatParam,
    /// }
    ///
    /// let params = Params::default();
    /// let mono_filter = || (pass() | params.cutoff.var() | dc(0.5f32)) >> lowpass();
    /// ```
    pub fn var(&self) -> An<Var> {
        var(&self.storage)
    }
}
