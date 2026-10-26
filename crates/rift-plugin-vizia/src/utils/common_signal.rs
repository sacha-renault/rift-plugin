//! common signals allow to create some signals at runtime.
//! Then users / views can access their values.
//!
//! The signals must be `installed` before anything else runs.

use std::ops::Deref;

use rift_plugin_types::transport::{BarInfo, BeatTime, LoopRegion, SecondsTime, TimeSignature};
use rift_plugin_utils::{DEFAULT_SAMPLERATE, DEFAULT_TEMPO};

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

            fn default_value() -> Self::Type {
                $default
            }
        }

        pub type $common_name = CommonSignal<$name>;
    };
}

mod __sealed {
    pub trait Sealed {}
}

pub trait CommonSignalKind: __sealed::Sealed + Copy + 'static {
    type Type: Copy + 'static;

    /// Value the signal holds between `install` and the first update from the
    /// host. Not a `const`: transport timestamps are only constructible through
    /// a non-const `From<f64>`.
    fn default_value() -> Self::Type;
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

    pub fn set(&self, v: T::Type) {
        self.write.set(v);
    }

    pub fn update<F: FnOnce(&mut T::Type)>(&self, f: F) {
        self.write.update(f);
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

    /// Retrieve the signal then get the value
    pub fn value(cx: &mut Context) -> T::Type {
        Self::signal(cx).get()
    }
}

impl<T: CommonSignalKind> Default for CommonSignal<T> {
    fn default() -> Self {
        Self::new(T::default_value())
    }
}

impl<T: CommonSignalKind> CommonSignal<T>
where
    T::Type: PartialEq,
{
    pub fn set_if_changed(&self, v: T::Type) {
        let equal = self
            .read
            .try_read_untracked()
            .map(|current| *current == v)
            .unwrap_or(false);

        if !equal {
            self.set(v);
        }
    }
}

declare_common_signal!(SamplerateSignal, _SampleRateInner, f64, DEFAULT_SAMPLERATE);
declare_common_signal!(FrameSignal, _FrameInner, (), ());

// Transport state: one signal per field, so a view only redraws for the value
// it actually reads.
declare_common_signal!(IsPlayingSignal, _IsPlayingInner, bool, false);
declare_common_signal!(IsRecordingSignal, _IsRecordingInner, bool, false);
declare_common_signal!(IsPrerollSignal, _IsPrerollInner, bool, false);

declare_common_signal!(SongPosBeatsSignal, _SongPosBeatsInner, BeatTime, 0.0.into());
declare_common_signal!(SongPosSecsSignal, _SongPosSecInner, SecondsTime, 0.0.into());

declare_common_signal!(TempoSignal, _TempoInner, f64, DEFAULT_TEMPO);
declare_common_signal!(TempoIncSignal, _TempoIncInner, f64, 0.0);

declare_common_signal!(LoopRegionSignal, _LoopRegionInner, Option<LoopRegion>, None);
declare_common_signal!(
    TimeSignatureSignal,
    _TimeSigInner,
    Option<TimeSignature>,
    None
);
declare_common_signal!(BarSignal, _BarInner, Option<BarInfo>, None);

impl FrameSignal {
    pub fn tick(&self) {
        self.set(());
    }
}
