use hug_shared::BlockTime;

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

    #[inline]
    pub fn time(&self) -> BlockTime {
        self.time.clone()
    }
}
