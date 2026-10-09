use vizia::vg::{Paint, PaintStyle};

use crate::dev_prelude::*;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Slots {
    pub track: Color,
    pub accent: Color,
    pub accent_end: Color,
    pub glow: Color,
}

impl Slots {
    pub fn read(cx: &DrawContext) -> Self {
        let accent = cx.font_color();
        let accent = if accent.a() == 0 {
            Color::rgb(139, 108, 255)
        } else {
            accent
        };
        let accent_end = cx.caret_color();
        Self {
            track: cx.background_color(),
            accent,
            // Fall back to a flat accent when no gradient end is given.
            accent_end: if accent_end.a() == 0 {
                accent
            } else {
                accent_end
            },
            glow: cx.selection_color(),
        }
    }
}

/// lerp two colors
pub(crate) fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let lerp = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color::rgba(
        lerp(a.r(), b.r()),
        lerp(a.g(), b.g()),
        lerp(a.b(), b.b()),
        lerp(a.a(), b.a()),
    )
}

/// A paint for anti-aliased fills.
pub(crate) fn fill_paint(color: Color) -> Paint {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(PaintStyle::Fill);
    paint.set_color(color);
    paint
}
