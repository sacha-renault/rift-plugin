use crate::dev_prelude::*;

mod button;
mod dial;
mod dropdown;
mod panel;
mod selector;
mod toggle;

use rift_plugin_params::{EnumParam, EnumValues};

pub use button::{ButtonExt, ButtonModifiers2};
pub use dial::{Dial, DialModifiers};
pub use dropdown::{Dropdown, DropdownModifiers};
pub use panel::Panel;
pub use selector::{Selector, SelectorModifiers};
pub use toggle::{Toggle, ToggleModifers};

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

pub fn param_dropdown<'a, E: EnumValues>(
    cx: &'a mut Context,
    param: &EnumParam<E>,
) -> Handle<'a, Dropdown> {
    let binding = ParamBinding::new(cx, param);
    let options = (0..E::count())
        .filter_map(E::from_index)
        .map(|variant| variant.to_string());
    let dropdown = Dropdown::new(cx, options, binding.normalized());
    binding.connect(dropdown)
}

/// An on/off switch
pub fn param_toggle<'a, P: Param>(cx: &'a mut Context, param: &P) -> Handle<'a, Toggle> {
    let binding = ParamBinding::new(cx, param);
    let toggle = Toggle::labeled(cx, binding.name(), binding.normalized());
    binding.connect(toggle)
}
