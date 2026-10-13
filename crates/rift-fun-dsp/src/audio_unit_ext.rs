use fundsp::prelude32::AudioUnit;
use rift_plugin_derive::modifiers;

#[modifiers(for T
    where
        T: AudioUnit + ?Sized,
        T: 'static,
)]
pub trait AudioUnitExt {
    #[concrete]
    /// Tick the AudioUnit to fill the rift frame.
    ///
    /// Two options:
    /// - unit has 0 input and N outputs: generate output and copy in frame
    /// - unit has N inputs and N outputs: copy frame in input > process > copy back in input
    fn fill_frame<'a, const N: usize>(&mut self, frame: &mut [f32; N]) {
        let inputs = self.inputs();
        let outputs = self.outputs();

        if inputs == 0 && outputs == N {
            let mut unit_output = [0f32; N];

            self.tick(&[], &mut unit_output);
            frame.copy_from_slice(&unit_output);
        } else if inputs == N && outputs == N {
            let mut unit_output = [0f32; N];
            let mut unit_input = [0f32; N];
            unit_input.copy_from_slice(frame);

            self.tick(&unit_input, &mut unit_output);
            frame.copy_from_slice(&unit_output);
        } else if cfg!(debug_assertions) {
            panic!("Use fill on a unit that has {inputs} / {outputs}")
        }
    }
}
