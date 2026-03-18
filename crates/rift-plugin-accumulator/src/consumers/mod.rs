use std::{cell::RefCell, rc::Rc};

use rift_plugin_shared::{
    prelude::ConsumerCell,
    transport::{BlockTime, ChannelsInfo},
};

mod audio_peaks;
mod spectrogram;
mod windowed_peaks;

/// A consumer that receives decoded audio blocks from an [`crate::prelude::AudioAccumulator`].
///
/// Implementors are called once per channel per drain cycle, receiving a
/// fixed-size slice of PCM samples along with the channel's position in the
/// bus and the block's transport timestamp.
///
/// # Call pattern
///
/// During a drain, `consume` is called once per channel in index order before
/// moving to the next block round. Use `channel_info.current` to demux
/// channels and `channel_info.total_channels` to know when a full frame
/// across all channels has been received.
pub trait AudioConsumer: 'static {
    /// Processes one block of PCM samples for a single channel.
    ///
    /// - `block` — interleaved f32 samples for the current channel only.
    ///   Length may be less than `N` for the final chunk of a render cycle.
    /// - `channel_info` — identifies which channel this block belongs to and
    ///   how many channels are in the bus in total.
    /// - `time` — transport position at the start of this block, or
    ///   [`BlockTime::none`] if timing information was unavailable.
    fn consume(&mut self, block: &[f32], channel_info: ChannelsInfo, time: BlockTime);
}

pub trait WrapsConsumer {
    fn wraps_consumer(self) -> ConsumerCell<Self>;
}

impl<T> WrapsConsumer for T
where
    T: AudioConsumer,
{
    fn wraps_consumer(self) -> ConsumerCell<Self> {
        Rc::new(RefCell::new(self))
    }
}

pub use audio_peaks::AudioPeaks;
pub use spectrogram::StftChannelConsumer;
pub use windowed_peaks::{Bucket, PeakBucket, WindowBuckets, WindowBucketsMode};
