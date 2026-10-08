use crate::dev_prelude::*;

mod dial;

pub use dial::{Dial, DialModifiers};

pub fn param_knob<'a, P: Param>(cx: &'a mut Context, param: &P) -> Handle<'a, Dial> {
    let binding = ParamBinding::new(cx, param);
    let dial = Dial::labeled(cx, binding.name(), binding.text(), binding.normalized());
    binding.connect(dial)
}
