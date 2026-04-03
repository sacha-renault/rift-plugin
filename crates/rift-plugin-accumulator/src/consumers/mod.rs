mod audio_peaks;
mod consumer_dispatcher;
mod spectrogram;
mod traits;
mod windowed_peaks;

pub use audio_peaks::AudioPeak;
pub use consumer_dispatcher::{ChannelMode, ConsumerDispatcher};
pub use spectrogram::StftConsumer;
pub use traits::{MonoConsumer, MultiConsumer};
pub use windowed_peaks::{Bucket, PeakBucket, WindowBuckets};
