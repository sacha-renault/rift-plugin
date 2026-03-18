use rift_plugin_core::transport::BlockTime;

pub struct TimedAudioBlock<const N: usize> {
    raw: [f32; N],
    slice_length: usize,

    /// This define the timing (seconds and beats) withing the song
    /// of the first beat of the BUFFER this block belongs to
    /// We might see many blocks with same seconds or beats if buffer_size > N
    time: BlockTime,
}

impl<const N: usize> TimedAudioBlock<N> {
    pub fn new(slice: &[f32], time: BlockTime) -> Self {
        let slice_length = slice.len();
        assert!(slice_length <= N);

        let mut raw = [0.0; N];
        raw[..slice_length].copy_from_slice(slice);
        TimedAudioBlock {
            raw,
            slice_length,
            time,
        }
    }

    pub fn as_slice(&self) -> &[f32] {
        &self.raw[..self.slice_length]
    }

    pub fn iter(&self) -> impl Iterator<Item = &f32> {
        self.as_slice().iter()
    }

    pub fn len(&self) -> usize {
        self.slice_length
    }

    pub fn is_empty(&self) -> bool {
        self.slice_length == 0
    }

    #[inline]
    pub fn time(&self) -> BlockTime {
        self.time
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_too_long_slice() {
        TimedAudioBlock::<10>::new(&[0.; 11], BlockTime::none());
    }

    #[test]
    fn test_correct_length() {
        let audio = vec![0.; 5];
        let block = TimedAudioBlock::<10>::new(&audio, BlockTime::none());
        assert_eq!(block.len(), audio.len());
        assert_eq!(block.as_slice().len(), audio.len());
        assert_eq!(block.time().beats(), None);
        assert!(!block.is_empty());
    }

    #[test]
    fn test_assert_iter() {
        fn with_iter<I: Iterator>(_: I) {}
        let audio = vec![0.; 5];
        let block = TimedAudioBlock::<10>::new(&audio, BlockTime::none());
        with_iter(block.iter())
    }
}
