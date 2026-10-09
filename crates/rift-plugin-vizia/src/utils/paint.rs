use vizia::prelude::{Color, DrawContext};
use vizia::vg::{
    self, BlurStyle, Color4f, MaskFilter, Paint, PaintCap, PaintStyle, Point, Shader, TileMode,
    gradient_shader::{Gradient, GradientColors, Interpolation},
};
use vizia::vg::{Matrix, Rect};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Slots {
    pub track: Color,
    pub accent: Color,
    pub accent_end: Color,
    pub glow: Color,
    pub rim: Color,
    pub marker: Color,
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
            rim: cx.border_color(),
            marker: cx.outline_color(),
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

pub(crate) fn square(center: Point, radius: f32) -> Rect {
    Rect::from_xywh(
        center.x - radius,
        center.y - radius,
        radius * 2.0,
        radius * 2.0,
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

/// A linear gradient between two points.
pub(crate) fn linear(a: Point, b: Point, from: Color, to: Color) -> Option<Shader> {
    let colors = [to_4f(from), to_4f(to)];
    vg::shaders::linear_gradient((a, b), &gradient(&colors, None), None)
}

/// A paint for anti-aliased strokes with round caps.
pub(crate) fn stroke_paint(width: f32) -> Paint {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(PaintStyle::Stroke);
    paint.set_stroke_width(width);
    paint.set_stroke_cap(PaintCap::Round);
    paint
}

pub(crate) fn blurred(mut paint: Paint, sigma: f32) -> Paint {
    paint.set_mask_filter(MaskFilter::blur(BlurStyle::Normal, sigma, false));
    paint
}

fn gradient<'a>(colors: &'a [Color4f], positions: Option<&'a [f32]>) -> Gradient<'a> {
    Gradient::new(
        GradientColors::new(colors, positions, TileMode::Clamp, None),
        Interpolation::default(),
    )
}

fn to_4f(color: Color) -> Color4f {
    Color4f::from(vg::Color::from(color))
}

pub(crate) fn sweep_gradient(
    center: Point,
    start: f32,
    span: f32,
    from: Color,
    to: Color,
) -> Option<Shader> {
    let colors = [from, to, to, from, from].map(to_4f);
    let end = (span / 360.0).clamp(0.01, 0.96);
    // `from` takes over 7 degrees before the circle closes, nearly instantly.
    let positions = [0.0, end, 0.98, 0.9801, 1.0];

    // Skia always starts a sweep at 3 o'clock: rotate the shader to move it.
    let rotation = Matrix::rotate_deg_pivot(start, center);
    vg::shaders::sweep_gradient(
        center,
        (0.0, 360.0),
        &gradient(&colors, Some(&positions)),
        &rotation,
    )
}
