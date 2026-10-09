use crate::widgets::*;
use rift_plugin_params::{EnumParam, EnumValues};

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
) -> Handle<'a, PopupSelector> {
    let binding = ParamBinding::new(cx, param);
    let options = (0..E::count())
        .filter_map(E::from_index)
        .map(|variant| variant.to_string());
    let dropdown = PopupSelector::new(cx, options, binding.normalized());
    binding.connect(dropdown)
}

/// An on/off switch
pub fn param_toggle<'a, P: Param>(cx: &'a mut Context, param: &P) -> Handle<'a, Toggle> {
    let binding = ParamBinding::new(cx, param);
    let toggle = Toggle::labeled(cx, binding.name(), binding.normalized());
    binding.connect(toggle)
}

pub fn param_slider<'a, P: Param>(cx: &'a mut Context, param: &P) -> Handle<'a, Fader> {
    let binding = ParamBinding::new(cx, param);
    let slider = Fader::labeled(cx, binding.name(), binding.text(), binding.normalized());
    binding.connect(slider)
}
