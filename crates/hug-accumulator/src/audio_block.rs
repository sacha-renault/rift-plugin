pub struct AudioBlock<const N: usize> {
    raw: [f32; N],
    slice_length: usize,
}

impl<const N: usize> AudioBlock<N> {
    pub fn new(slice: &[f32]) -> Self {
        let slice_length = slice.len();
        assert!(slice_length <= N);

        let mut raw = [0.0; N];
        raw[..slice_length].copy_from_slice(slice);
        AudioBlock { raw, slice_length }
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
}
