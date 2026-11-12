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

/// A vizia [`Select`] showing the variants of an enum parameter.
///
/// `Select` isn't a [`Control`], so the parameter gesture is driven here
/// instead of through [`ParamBinding::connect`].
pub fn param_select<'a, E: EnumValues>(
    cx: &'a mut Context,
    param: &EnumParam<E>,
) -> Handle<'a, Select> {
    let binding = ParamBinding::new(cx, param);
    let options: Vec<String> = (0..E::count())
        .filter_map(E::from_index)
        .map(|variant| variant.to_string())
        .collect();
    let count = options.len();

    let value = binding.normalized();
    let selected = value.map(move |v| Some(index_of(*v, count)));

    Select::new(cx, Signal::new(options), selected, true)
        .on_select(move |cx, index| {
            if index_of(value.get_untracked(), count) != index {
                binding.begin(cx);
                binding.set(cx, value_of(index, count));
                binding.end(cx);
            }
        })
        .on_mouse_down(move |cx, button| {
            if matches!(button, MouseButton::Right) {
                binding.open_context_menu(cx);
            }
        })
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

/// A tick box
pub fn param_check_box<'a, P: Param>(cx: &'a mut Context, param: &P) -> Handle<'a, CheckBox> {
    let binding = ParamBinding::new(cx, param);
    let check_box = CheckBox::labeled(cx, binding.name(), binding.normalized());
    binding.connect(check_box)
}
