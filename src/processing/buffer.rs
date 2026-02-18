pub struct Buffers<'a> {
    pub main: Buffer<'a>,
    pub aux_inputs: Option<Buffer<'a>>,
    pub aux_output: Option<Buffer<'a>>,
}

impl<'a> Buffers<'a> {
    pub(crate) fn new(main: &'a Vec<&'a mut [f32]>) -> Self {
        Self {
            main: Buffer::new(main),
            aux_inputs: None,
            aux_output: None,
        }
    }

    // pub(crate) fn with_inputs(mut self, aux_inputs: &'a  mut Vec<&'a mut [f32]>) -> Self {
    //     self.aux_inputs = Some(Buffer::new(aux_inputs));
    //     self
    // }

    // pub(crate) fn with_outputs(mut self, aux_output: &'a  mut Vec<&'a mut [f32]>) -> Self {
    //     self.aux_output = Some(Buffer::new(aux_output));
    //     self
    // }
}

pub struct Buffer<'a> {
    vec: &'a Vec<&'a mut [f32]>,
}

impl<'a> Buffer<'a> {
    #[inline]
    pub(crate) fn new(buffer: &'a Vec<&'a mut [f32]>) -> Self {
        Self { vec: buffer }
    }

    #[inline]
    pub fn channels(&self) -> usize {
        self.vec.len()
    }

    #[inline]
    pub fn samples(&self) -> usize {
        if self.channels() == 0 {
            0
        } else {
            // We assume all the buffers as same length ...
            // Otherwise that's fucked up
            self.vec[0].len()
        }
    }

    pub fn iter_channels(&'a self) -> ChannelsIterator<'a> {
        let samples = self.samples();
        let channels = self.channels();
        ChannelsIterator {
            vec: self.vec,
            position: 0,
            channels,
            samples,
        }
    }
}

/// This struct iter over the buffer, yielding all
/// channels at time n
/// Ex: [
///     [1,2,3],
///     [4,5,6]
/// ]
/// Will yield [(1, 4), (2, 5), (3, 6)]
pub struct ChannelsIterator<'a> {
    vec: &'a Vec<&'a mut [f32]>,
    position: usize,
    channels: usize,
    samples: usize,
}

impl<'a> Iterator for ChannelsIterator<'a> {
    type Item = SamplesIterator<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.samples {
            let item = Some(SamplesIterator {
                vec: self.vec,
                position: 0,
                channel_position: self.position,
                channels: self.channels,
            });
            self.position += 1;
            item
        } else {
            None
        }
    }
}

pub struct SamplesIterator<'a> {
    vec: &'a Vec<&'a mut [f32]>,
    channel_position: usize,
    position: usize,
    channels: usize,
}

impl<'a> Iterator for SamplesIterator<'a> {
    type Item = &'a mut f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.channels {
            let position = self.position;
            self.position += 1;
            unsafe {
                let channel_slice = (*self.vec).as_ptr().add(position) as *mut &mut [f32];
                Some(&mut (*channel_slice)[self.channel_position])
            }
        } else {
            None
        }
    }
}
