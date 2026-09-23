use fundsp::prelude::*;
use rift_plugin_buffers::Buffer;

pub trait AudioUnitRiftProcess {
    fn process_rift_buffer(&mut self, size: usize, buffer: &mut Buffer);
}

impl<T> AudioUnitRiftProcess for T
where
    T: AudioNode,
{
    fn process_rift_buffer(&mut self, size: usize, buffer: &mut Buffer) {
        // The default implementation is a fallback that calls into `tick`.
        debug_assert!(size <= MAX_BUFFER_SIZE);
        debug_assert_eq!(buffer.channels(), self.inputs());
        debug_assert_eq!(buffer.channels(), self.outputs());

        // Note. We could build `tick` inputs from `[f32; 8]` or `f32x8` temporary
        // values to make index arithmetic easier but according to benchmarks
        // it doesn't make a difference.
        let mut input_frame: Frame<f32, <Self as AudioNode>::Inputs> = Frame::default();

        let mut end = 0;
        let samples = buffer.samples();
        let raw_data = buffer.raw_ptrs();
        while end < samples {
            let next_end = std::cmp::max(samples, end + size);
        }
    }
}
