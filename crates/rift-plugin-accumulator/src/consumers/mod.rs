use std::{cell::RefCell, rc::Rc};

use rift_plugin_core::prelude::ConsumerCell;

mod audio_peaks;
mod consumer_dispatcher;
mod spectrogram;
mod traits;
mod windowed_peaks;

pub use audio_peaks::AudioPeaks;
pub use consumer_dispatcher::{ChannelMode, ConsumerDispatcher};
pub use spectrogram::StftConsumer;
pub use traits::{MonoConsumer, MultiConsumer};
pub use windowed_peaks::{Bucket, PeakBucket, WindowBuckets};

pub trait WrapsConsumer {
    /// Wraps an [`AudioConsumer`] into a [`ConsumerCell`].
    ///
    /// **Notes**:
    /// [`ConsumerCell`] is a pretty name for [`Rc<RefCell<_>>`]
    fn wraps_consumer(self) -> ConsumerCell<Self>;
}

impl<T> WrapsConsumer for T
where
    T: MonoConsumer,
{
    fn wraps_consumer(self) -> ConsumerCell<Self> {
        Rc::new(RefCell::new(self))
    }
}
