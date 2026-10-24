mod audio_peaks;
mod consumer_dispatcher;
mod rolling_window;
mod spectrogram;
mod traits;

pub use audio_peaks::AudioPeak;
pub use consumer_dispatcher::{ConsumerDispatcher, MonoConsumerKind};
pub use rolling_window::{Bucket, PeakBucket, RmsBucket, RollingWindow};
pub use spectrogram::StftConsumer;
pub use traits::{Consumer, MonoConsumer, MultiConsumer};
