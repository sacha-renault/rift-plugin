use clack_extensions::params::ParamInfoFlags;

use super::traits::ClapParam;

/// A zero-cost wrapper around a raw pointer to a [`ClapParam`].
/// The underlying parameter lives in the host and outlives this wrapper.
#[derive(Clone, Copy)]
pub struct ParamPtr {
    /// The shared struct (and thus the params) lives longer
    /// than the program and is handled by the host.
    /// We can safely deref those params and make it much easier for
    /// gui to use any params
    ptr: *const dyn ClapParam,
}

impl ParamPtr {
    pub fn new(ptr: *const dyn ClapParam) -> Self {
        Self { ptr }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use clack_plugin::prelude::ClapId;

    struct MockParam {
        value: std::cell::Cell<f64>,
    }

    impl MockParam {
        fn new(value: f64) -> Self {
            Self {
                value: std::cell::Cell::new(value),
            }
        }
    }

    impl ClapParam for MockParam {
        fn id(&self) -> ClapId {
            ClapId::from(1u32)
        }
        fn name(&self) -> &str {
            "mock"
        }
        fn module(&self) -> &str {
            "test/module"
        }
        fn unit(&self) -> &str {
            "dB"
        }
        fn get_raw(&self) -> f64 {
            self.value.get()
        }
        fn set_raw(&self, value: f64) {
            self.value.set(value);
        }
        fn default_raw(&self) -> f64 {
            0.5
        }
        fn get_normalized(&self) -> f64 {
            self.normalize(self.value.get())
        }
        fn set_normalized(&self, normalized: f64) {
            self.value.set(self.denormalize(normalized));
        }
        fn normalize(&self, value: f64) -> f64 {
            value / 100.0
        }
        fn denormalize(&self, normalized: f64) -> f64 {
            normalized * 100.0
        }
        fn min_value(&self) -> f64 {
            0.0
        }
        fn max_value(&self) -> f64 {
            100.0
        }
        fn flags(&self) -> ParamInfoFlags {
            ParamInfoFlags::empty()
        }
        fn as_ptr(&self) -> ParamPtr {
            ParamPtr::new(self as *const dyn ClapParam)
        }
    }

    fn make_ptr(mock: &MockParam) -> ParamPtr {
        ParamPtr::new(mock as *const dyn ClapParam)
    }

    #[test]
    fn test_passthrough_metadata() {
        let mock = MockParam::new(42.0);
        let ptr = make_ptr(&mock);
        assert_eq!(ptr.name(), "mock");
        assert_eq!(ptr.module(), "test/module");
        assert_eq!(ptr.unit(), "dB");
    }

    #[test]
    fn test_get_set_raw() {
        let mock = MockParam::new(10.0);
        let ptr = make_ptr(&mock);
        assert_eq!(ptr.get_raw(), 10.0);
        ptr.set_raw(99.0);
        assert_eq!(ptr.get_raw(), 99.0);
    }

    #[test]
    fn test_normalize_denormalize() {
        let mock = MockParam::new(50.0);
        let ptr = make_ptr(&mock);
        assert_eq!(ptr.normalize(50.0), 0.5);
        assert_eq!(ptr.denormalize(0.5), 50.0);
    }

    #[test]
    fn test_normalized_get_set() {
        let mock = MockParam::new(50.0);
        let ptr = make_ptr(&mock);
        assert_eq!(ptr.get_normalized(), 0.5);
        ptr.set_normalized(1.0);
        assert_eq!(ptr.get_raw(), 100.0);
    }

    #[test]
    fn test_value_to_text() {
        let mock = MockParam::new(3.14);
        let ptr = make_ptr(&mock);
        let text = ptr.to_text();
        assert_eq!(&text, "3.14dB");
    }

    #[test]
    fn test_text_to_value() {
        let mock = MockParam::new(0.0);
        let ptr = make_ptr(&mock);
        let cstr = std::ffi::CString::new("42.0dB").unwrap();
        assert_eq!(ptr.text_to_value(&cstr), Some(42.0));
    }

    #[test]
    fn test_text_to_value_no_unit() {
        let mock = MockParam::new(0.0);
        let ptr = make_ptr(&mock);
        let cstr = std::ffi::CString::new("42.0").unwrap();
        assert_eq!(ptr.text_to_value(&cstr), None);
    }

    #[test]
    fn test_min_max_default() {
        let mock = MockParam::new(0.0);
        let ptr = make_ptr(&mock);
        assert_eq!(ptr.min_value(), 0.0);
        assert_eq!(ptr.max_value(), 100.0);
        assert_eq!(ptr.default_raw(), 0.5);
    }

    #[test]
    fn test_as_ptr_roundtrip() {
        let mock = MockParam::new(77.0);
        let ptr = make_ptr(&mock);
        let ptr2 = ptr.as_ptr();
        assert_eq!(ptr2.get_raw(), 77.0);
    }

    #[test]
    fn test_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ParamPtr>();
    }

    #[test]
    fn test_id_flags_ptr() {
        let mock = MockParam::new(77.0);

        assert_eq!(mock.id(), ClapId::from(1u32));
        assert_eq!(mock.flags(), ParamInfoFlags::empty());
    }

    #[test]
    fn test_param_info() {
        let mock = MockParam::new(77.0);
        let infos = mock.param_info();

        assert_eq!(infos.id, ClapId::from(1u32));
    }
}
