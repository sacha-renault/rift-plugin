//! common signals allow to create some signals at runtime.
//! Then users / views can access their values.
//!
//! The signals must be `installed` before anything else runs.

use std::ops::Deref;

use rift_plugin_utils::DEFAULT_SAMPLERATE;

use crate::dev_prelude::*;

macro_rules! declare_common_signal {
    ($common_name:ident, $name:ident, $type:ty, $default:expr) => {
        // ZST struct
        #[derive(Clone, Copy)]
        #[doc(hidden)]
        pub struct $name;
        impl __sealed::Sealed for $name {}

        impl CommonSignalKind for $name {
            type Type = $type;

            const DEFAULT: Self::Type = $default;
        }

        pub type $common_name = CommonSignal<$name>;
    };
}

mod __sealed {
    pub trait Sealed {}
}

pub trait CommonSignalKind: __sealed::Sealed + Copy + 'static {
    type Type: Copy + 'static;

    const DEFAULT: Self::Type;
}

#[derive(Clone, Copy)]
pub struct CommonSignal<T: CommonSignalKind> {
    read: ReadSignal<T::Type>,
    write: WriteSignal<T::Type>,
}

impl<T: CommonSignalKind> Model for CommonSignal<T> {}

impl<T: CommonSignalKind> Deref for CommonSignal<T> {
    type Target = ReadSignal<T::Type>;

    fn deref(&self) -> &Self::Target {
        &self.read
    }
}

impl<T: CommonSignalKind> CommonSignal<T> {
    pub(crate) fn new(v: T::Type) -> Self {
        let base = Signal::new(v);
        Self {
            write: base.write_only(),
            read: base.read_only(),
        }
    }

    pub(crate) fn set(&self, v: T::Type) {
        self.write.set(v);
    }

    /// Install the signal on the root entity
    pub(crate) fn install(self, cx: &mut Context) {
        cx.with_current(Entity::root(), |cx| {
            self.build(cx);
        });
    }

    /// Retreive the signal on the root entity
    pub fn signal(cx: &mut Context) -> Self {
        cx.with_current(Entity::root(), |cx| {
            *cx.try_data::<Self>()
                .expect("CommonSignal wasn't install in the app")
        })
    }
}

impl<T: CommonSignalKind> Default for CommonSignal<T> {
    fn default() -> Self {
        Self::new(T::DEFAULT)
    }
}

impl<T: CommonSignalKind> CommonSignal<T>
where
    T::Type: PartialEq,
{
    pub fn set_if_changed(&self, v: T::Type) {
        if v != *self.read.read() {
            self.set(v);
        }
    }
}

declare_common_signal!(SamplerateSignal, _SrSignalInner, f64, DEFAULT_SAMPLERATE);
declare_common_signal!(FrameSignal, _FrameSignalInner, (), ());

impl FrameSignal {
    pub fn tick(&self) {
        self.set(());
    }
}
