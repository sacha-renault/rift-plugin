use clack_extensions::params::ParamInfoFlags;

use crate::prelude::ClapParam;

/// A zero-cost wrapper around a raw pointer to a [`ClapParam`].
/// The underlying parameter lives in the host and outlives this wrapper.
#[derive(Clone, Copy)]
pub struct ParamPtr {
    /// The shared struct (and thus the params) lives longer
    /// than the program and is handled by the host.
    /// We can safely deref those params and make it much easier for
    /// gui to use any params
    pub(crate) ptr: *const dyn ClapParam,
}

unsafe impl Send for ParamPtr {}
unsafe impl Sync for ParamPtr {}

impl ClapParam for ParamPtr {
    #[inline]
    fn id(&self) -> clack_plugin::prelude::ClapId {
        unsafe { (*self.ptr).id() }
    }

    #[inline]
    fn name(&self) -> &str {
        unsafe { (*self.ptr).name() }
    }

    #[inline]
    fn module(&self) -> &str {
        unsafe { (*self.ptr).module() }
    }

    #[inline]
    fn unit(&self) -> &str {
        unsafe { (*self.ptr).unit() }
    }

    #[inline]
    fn get_raw(&self) -> f64 {
        unsafe { (*self.ptr).get_raw() }
    }

    #[inline]
    fn set_raw(&self, value: f64) {
        unsafe { (*self.ptr).set_raw(value) }
    }

    #[inline]
    fn default_raw(&self) -> f64 {
        unsafe { (*self.ptr).default_raw() }
    }

    #[inline]
    fn get_normalized(&self) -> f64 {
        unsafe { (*self.ptr).get_normalized() }
    }

    #[inline]
    fn set_normalized(&self, normalized: f64) {
        unsafe { (*self.ptr).set_normalized(normalized) }
    }

    // Display formatting
    #[inline]
    fn value_to_text(&self, value: f64, writer: &mut dyn core::fmt::Write) -> std::fmt::Result {
        unsafe { (*self.ptr).value_to_text(value, writer) }
    }

    #[inline]
    fn text_to_value(&self, value: &std::ffi::CStr) -> Option<f64> {
        unsafe { (*self.ptr).text_to_value(value) }
    }

    #[inline]
    fn flags(&self) -> ParamInfoFlags {
        unsafe { (*self.ptr).flags() }
    }

    #[inline]
    fn normalize(&self, value: f64) -> f64 {
        unsafe { (*self.ptr).normalize(value) }
    }

    #[inline]
    fn denormalize(&self, normalized: f64) -> f64 {
        unsafe { (*self.ptr).denormalize(normalized) }
    }

    #[inline]
    fn min_value(&self) -> f64 {
        unsafe { (*self.ptr).min_value() }
    }

    #[inline]
    fn max_value(&self) -> f64 {
        unsafe { (*self.ptr).max_value() }
    }

    fn as_ptr(&self) -> ParamPtr {
        *self
    }
}
