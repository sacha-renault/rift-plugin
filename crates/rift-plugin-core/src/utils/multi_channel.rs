use std::ops::{Index, IndexMut};

/// A fixed-size collection of per-channel state.
///
/// [`MultiChannel<T>`] holds one `T` per audio channel, created at construction
/// time via a factory closure. Individual channels are accessed through
/// callback-based methods that pass a shared or mutable reference to the
/// underlying `T`.
///
/// This is typically paired with a `MonoConsumer` (available in accumulator package) implementation so that
/// a single-channel processor can be used in a multi-channel context
/// without any awareness of channel routing.
///
/// # Examples
///
/// ```ignore
/// let peaks = MultiChannel::new(2, AudioPeaks::new);
///
/// // Read the smoothed peak of channel 0
/// let level = peaks.with_channel(0, |p| p.peak());
///
/// // Safely handle an out-of-bounds channel
/// let maybe = peaks.try_with_channel(5, |p| p.peak()); // None
/// ```
pub struct MultiChannel<T> {
    channels: Vec<T>,
}

impl<T> MultiChannel<T> {
    /// Creates a new `MultiChannel` with `channels_count` independent instances
    /// of `T`, each produced by calling `factory`.
    pub fn new<F>(channels_count: usize, factory: F) -> Self
    where
        F: FnMut() -> T,
    {
        let mut channels = Vec::new();
        channels.resize_with(channels_count, factory);
        Self { channels }
    }

    /// Calls `func` with a shared reference to the channel at `channel`.
    ///
    /// # Panics
    ///
    /// Panics if `channel` is out of bounds.
    pub fn with_channel<F, R>(&self, channel: usize, func: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        func(&self.channels[channel])
    }

    /// Calls `func` with a shared reference to the channel at `channel`,
    /// returning `None` if `channel` is out of bounds.
    pub fn try_with_channel<F, R>(&self, channel: usize, func: F) -> Option<R>
    where
        F: FnOnce(&T) -> R,
    {
        self.channels.get(channel).map(func)
    }

    /// Calls `func` with a mutable reference to the channel at `channel`.
    ///
    /// # Panics
    ///
    /// Panics if `channel` is out of bounds.
    pub fn with_channel_mut<F, R>(&mut self, channel: usize, func: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        func(&mut self.channels[channel])
    }

    /// Calls `func` with a mutable reference to the channel at `channel`,
    /// returning `None` if `channel` is out of bounds.
    pub fn try_with_channel_mut<F, R>(&mut self, channel: usize, func: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> R,
    {
        self.channels.get_mut(channel).map(func)
    }

    /// Fold on all channels. Same as [`Iterator::fold`].
    pub fn fold<F, B>(&self, init: B, func: F) -> B
    where
        F: FnMut(B, &T) -> B,
    {
        self.channels.iter().fold(init, func)
    }

    /// Calls `func` on ALL channels.
    pub fn apply_all<F>(&mut self, func: F)
    where
        F: FnMut(&mut T),
    {
        self.channels.iter_mut().for_each(func);
    }

    /// Iterator over the channels
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.channels.iter()
    }

    /// Mutable iterator over the channels
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.channels.iter_mut()
    }

    /// Returns the number of channels this instance was created with.
    pub fn channels(&self) -> usize {
        self.channels.len()
    }
}

impl<T> Index<usize> for MultiChannel<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.channels[index]
    }
}

impl<T> IndexMut<usize> for MultiChannel<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.channels[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_channel() {
        let mc = MultiChannel::new(5, || 3i32);
        assert_eq!(mc.with_channel(0, |c| *c), 3);
        assert_eq!(mc.channels(), 5);
    }

    #[test]
    #[should_panic]
    fn with_channel_out_of_bound() {
        let mc = MultiChannel::new(5, || 3i32);
        mc.with_channel(5, |_| ());
    }

    #[test]
    fn try_with_channel() {
        let mc = MultiChannel::new(5, || 3i32);
        assert_eq!(mc.try_with_channel(0, |c| *c), Some(3))
    }

    #[test]
    fn try_with_channel_out_of_bounds() {
        let mc = MultiChannel::new(5, || 3i32);
        assert!(mc.try_with_channel(5, |c| *c).is_none())
    }

    #[test]
    fn with_channel_mut() {
        let mut mc = MultiChannel::new(5, || 3i32);
        assert_eq!(mc.with_channel(0, |c| *c), 3);
        mc.with_channel_mut(0, |c| *c = 5);
        assert_eq!(mc.with_channel(0, |c| *c), 5);
    }

    #[test]
    fn try_with_channel_mut() {
        let mut mc = MultiChannel::new(5, || 3i32);
        assert_eq!(mc.try_with_channel_mut(0, |c| *c), Some(3));
        mc.with_channel_mut(0, |c| *c = 5);
        assert_eq!(mc.try_with_channel_mut(0, |c| *c), Some(5));
    }

    #[test]
    fn fold_all_channels() {
        let mc = MultiChannel::new(5, || 3i32);
        assert_eq!(mc.fold(0, |acc, c| acc + *c), 15);
    }

    #[test]
    fn apply_all_channels() {
        let mut mc = MultiChannel::new(5, || 3i32);

        for c in mc.iter() {
            assert_eq!(*c, 3);
        }

        mc.apply_all(|c| *c = 5);

        for c in mc.iter() {
            assert_eq!(*c, 5);
        }
    }

    #[test]
    fn iter_mut() {
        let mut mc = MultiChannel::new(5, || 3i32);

        for c in mc.iter() {
            assert_eq!(*c, 3);
        }

        for c in mc.iter_mut() {
            *c = 5;
        }

        for i in 0..5 {
            assert_eq!(mc[i], 5);
        }
    }

    #[test]
    fn index_mut() {
        let mut mc = MultiChannel::new(5, || 3i32);

        for c in mc.iter() {
            assert_eq!(*c, 3);
        }

        for i in 0..5 {
            mc[i] = 5;
        }

        for i in 0..5 {
            assert_eq!(mc[i], 5);
        }
    }
}
