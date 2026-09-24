use fundsp::prelude32::AudioUnit;
use rift_plugin_buffers::Frame;

pub trait AudioUnitExt {
    fn fill_frame<'a, const N: usize>(&mut self, frame: Frame<'a>);
}

impl<T> AudioUnitExt for T
where
    T: AudioUnit + ?Sized,
{
    fn fill_frame<'a, const N: usize>(&mut self, frame: Frame<'a>) {
        let inputs = self.inputs();
        let outputs = self.outputs();

        if inputs == 0 && outputs == N {
            let mut unit_output = [0f32; N];
            self.tick(&[], &mut unit_output);

            for (sample, temp) in frame.zip(&mut unit_output) {
                *sample = *temp;
            }
        } else if cfg!(debug_assertions) {
            panic!("Use fill on a unit that has {inputs} / {outputs}")
        }
    }
}
