use crate::dev_prelude::*;

#[derive(Clone, Copy)]
pub struct ParamBinding {
    ptr: ParamPtr,
    value: Signal<f32>,
}

impl ParamBinding {
    // Binds `param`, registering it with the GUI if needed.
    ///
    /// Several bindings of the same parameter share one signal.
    pub fn new<P: Param>(cx: &mut Context, param: &P) -> Self {
        Self {
            ptr: param.as_ptr(),
            value: register_param(cx, param),
        }
    }

    pub fn id(&self) -> ClapId {
        self.ptr.id()
    }

    pub fn name(&self) -> String {
        self.ptr.name().to_owned()
    }

    pub fn unit(&self) -> &str {
        self.ptr.unit()
    }

    pub fn normalized(&self) -> Signal<f32> {
        self.value
    }

    pub fn default_normalized(&self) -> f32 {
        self.ptr.normalize(self.ptr.default_plain())
    }

    pub fn format(&self, normalized: f32) -> String {
        let plain = self.ptr.denormalize(normalized.clamp(0.0, 1.0));
        let plain = round_for_display(plain, self.ptr.min_value(), self.ptr.max_value());

        let mut text = String::new();
        if self.ptr.value_to_text(plain, &mut text).is_err() {
            let unit = self.unit();
            text = format!("{plain} {unit}");
        }
        text
    }

    pub fn text(&self) -> Memo<String> {
        let (binding, value) = (*self, self.value);
        Memo::new(move |_| binding.format(value.get()))
    }

    pub fn begin(&self, cx: &mut impl EmitContext) {
        cx.emit_to(Entity::root(), GuiParamEvent::gesture_start(self.id()));
    }

    pub fn set(&self, cx: &mut impl EmitContext, normalized: f32) {
        let normalized = normalized.clamp(0.0, 1.0);
        self.value.set(normalized);
        cx.emit_to(
            Entity::root(),
            GuiParamEvent::value(self.id(), self.ptr.denormalize(normalized)),
        );
    }

    pub fn end(&self, cx: &mut impl EmitContext) {
        cx.emit_to(Entity::root(), GuiParamEvent::gesture_end(self.id()));
    }

    pub fn open_context_menu(&self, cx: &mut EventContext) {
        let scale = cx.scale_factor().max(f32::EPSILON);
        let mouse = cx.mouse();
        let request = ParamContextMenuRequest {
            id: self.id(),
            x: (mouse.cursor_x / scale).round() as i32,
            y: (mouse.cursor_y / scale).round() as i32,
            screen: 0,
        };
        cx.emit_to(Entity::root(), request);
    }

    /// Wires a control to the parameter: its edits are reported to the host as
    /// gestures, and its default value is the one of the parameter.
    ///
    /// The control must be showing [`normalized`] values.
    pub fn connect<C: ControlExt>(&self, control: C) -> C {
        let binding = *self;
        control
            .default_normalized(self.default_normalized())
            .on_begin(move |cx| binding.begin(cx))
            .on_change(move |cx, value| binding.set(cx, value))
            .on_end(move |cx| binding.end(cx))
            .on_context_menu(move |cx| binding.open_context_menu(cx))
    }
}

/// Round the value for display so it's never too long with too many decimals
fn round_for_display(value: f32, min: f32, max: f32) -> f32 {
    let span = (max - min).abs();
    if !value.is_finite() || span <= f32::EPSILON {
        return value;
    }

    let decimals = (-(span / 200.0).log10()).ceil().clamp(0.0, 4.0) as i32;
    let factor = 10f32.powi(decimals);
    let rounded = (value * factor).round() / factor;

    // Never display "-0".
    if rounded == 0.0 { 0.0 } else { rounded }
}
