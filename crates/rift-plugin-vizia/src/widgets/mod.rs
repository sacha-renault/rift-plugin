use crate::dev_prelude::*;

mod dial;
mod panel;
mod selector;

use rift_plugin_params::{EnumParam, EnumValues};

pub use dial::{Dial, DialModifiers};
pub use panel::Panel;
pub use selector::{Selector, SelectorModifiers};

pub fn param_knob<'a, P: Param>(cx: &'a mut Context, param: &P) -> Handle<'a, Dial> {
    let binding = ParamBinding::new(cx, param);
    let dial = Dial::labeled(cx, binding.name(), binding.text(), binding.normalized());
    binding.connect(dial)
}

pub fn param_selector<'a, E: EnumValues>(
    cx: &'a mut Context,
    param: &EnumParam<E>,
) -> Handle<'a, Selector> {
    let binding = ParamBinding::new(cx, param);
    let options = (0..E::count())
        .filter_map(E::from_index)
        .map(|variant| variant.to_string());
    let selector = Selector::new(cx, options, binding.normalized());
    binding.connect(selector)
}
