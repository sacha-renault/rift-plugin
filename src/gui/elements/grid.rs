use rift_plugin_shared::utils::spaces::Linespace;
use vizia::vg;

use super::gui_prelude::*;

/// Defines the mathematical distribution of lines across the grid.
pub enum GridScale {
    /// Equally spaced intervals.
    Linear { start: f32, end: f32, count: usize },
    /// Exponentially spaced intervals (e.g., frequency scales).
    Logarithmic {
        start: f32,
        end: f32,
        base: f32,
        sub_ticks: usize,
    },
    /// Arbitrary positions defined by the caller.
    Lines(Vec<(f32, f32)>),
}

impl GridScale {
    /// Creates a linear grid with equally spaced ticks.
    pub fn linear(start: f32, end: f32, count: usize) -> Self {
        Self::Linear { start, end, count }
    }

    /// Creates a logarithmic grid.
    ///
    /// # Panics
    /// Panics if `base` is less than or equal to 1.0,
    /// or if the range includes non-positive values.
    pub fn logarithmic(start: f32, end: f32, base: f32, sub_ticks: usize) -> Self {
        Self::Logarithmic {
            start,
            end,
            base,
            sub_ticks,
        }
    }

    /// Creates a grid with a single line at the given normalized position and value.
    pub fn line(normalized: f32, value: f32) -> Self {
        Self::Lines(vec![(normalized, value)])
    }

    /// Creates a grid from multiple manually defined lines.
    pub fn lines(lines: Vec<(f32, f32)>) -> Self {
        Self::Lines(lines)
    }

    fn compile(&self) -> CompiledGridScale {
        match self {
            GridScale::Linear { start, end, count } => self.compile_linear(*start, *end, *count),
            GridScale::Logarithmic {
                start,
                end,
                base,
                sub_ticks,
            } => self.compile_log(*start, *end, *base, *sub_ticks),
            GridScale::Lines(lines) => self.compile_manual_lines(lines),
        }
    }

    fn compile_linear(&self, start: f32, end: f32, count: usize) -> CompiledGridScale {
        if count < 2 || start == end {
            return CompiledGridScale(vec![]);
        }

        let range = end - start;
        let values = Linespace::new(0.0, 1.0, count)
            .map(|t| GridValue {
                normalized: t,
                value: start + t * range,
            })
            .collect();

        CompiledGridScale(values)
    }

    fn compile_log(&self, start: f32, end: f32, base: f32, sub_ticks: usize) -> CompiledGridScale {
        if start <= 0.0 || end <= 0.0 || start >= end || base <= 1.0 {
            return CompiledGridScale(vec![]);
        }

        let mut values = Vec::new();
        let log_start = start.log(base);
        let log_range = end.log(base) - log_start;

        let mut current_major = start;
        while current_major <= end {
            // Add major tick
            self.push_log_value(&mut values, current_major, log_start, log_range, base);

            // Add sub-ticks between this major and the next
            let next_major = current_major * base;
            if sub_ticks > 0 {
                let step = (next_major - current_major) / (sub_ticks as f32 + 1.0);
                for i in 1..=sub_ticks {
                    let sub_val = current_major + step * (i as f32);
                    if sub_val > end {
                        break;
                    }
                    self.push_log_value(&mut values, sub_val, log_start, log_range, base);
                }
            }
            current_major = next_major;
        }

        CompiledGridScale(values)
    }

    fn compile_manual_lines(&self, positions: &[(f32, f32)]) -> CompiledGridScale {
        let values = positions
            .iter()
            .map(|&(n, v)| GridValue {
                normalized: n.clamp(0.0, 1.0),
                value: v,
            })
            .collect();
        CompiledGridScale(values)
    }

    fn push_log_value(
        &self,
        storage: &mut Vec<GridValue>,
        val: f32,
        l_start: f32,
        l_range: f32,
        base: f32,
    ) {
        let t = (val.log(base) - l_start) / l_range;
        if t >= -0.0001 && t <= 1.0001 {
            storage.push(GridValue {
                normalized: t.clamp(0.0, 1.0),
                value: val,
            });
        }
    }
}

/// Compile user [`GridScale`] input so we never have to recompute values
/// (i hate allocation please stop)
struct CompiledGridScale(Vec<GridValue>);

/// A single tick definition in the compiled grid.
struct GridValue {
    normalized: f32,

    // ALLOW:
    // We might later plot the value
    // So we keep this here for now
    #[allow(unused)]
    value: f32,
}

/// A helper to draw grid lines on a [`Plot`] or similar canvas.
///
/// # Example:
/// ```ignore
/// ZStack::new(cx, |cx| {
/// // Create a logarithmic horizontal frequency grid (20Hz to 20kHz)
/// PlotGrid::new(cx, GridScale::logarithmic(20.0, 20000.0, 10.0, 8))
/// .orientation(Orientation::Horizontal)
///     .color(Color::gray())
///     .opacity(0.2);
///
///     // Create a linear vertical decibel grid (-12dB to +12dB)
/// PlotGrid::new(cx, GridScale::linear(-12.0, 12.0, 5))
/// .orientation(Orientation::Vertical)
///     .color(Color::gray())
///     .opacity(0.2);
/// });
/// ```
///
/// # Note
/// Only redraws when the underlying plot's layout changes (handled by the parent `View` impl).
#[derive(HandleExtension)]
pub struct PlotGrid {
    axe_values: CompiledGridScale,

    #[extension(ext)]
    orientation: Orientation,
}

impl View for PlotGrid {
    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let mut paint = vg::Paint::default();
        paint.set_color(cx.font_color());
        paint.set_stroke_width(cx.border_width());
        paint.set_stroke_cap(vg::PaintCap::Square);
        paint.set_style(vg::PaintStyle::Stroke);

        let viewport_transform = ViewportTransform::new(cx.bounds());

        for axe_val in &self.axe_values.0 {
            let pos = axe_val.normalized;

            let (start, end) = match self.orientation {
                // X-Axis: Draw vertical lines spanning from top to bottom
                Orientation::Horizontal => (
                    viewport_transform.transform(pos, 0.0),
                    viewport_transform.transform(pos, 1.0),
                ),
                // Y-Axis: Draw horizontal lines spanning from left to right
                Orientation::Vertical => (
                    viewport_transform.transform(0.0, pos),
                    viewport_transform.transform(1.0, pos),
                ),
            };

            canvas.draw_line(start, end, &paint);
        }
    }
}

impl PlotGrid {
    /// Creates a new grid based on the provided scale.
    pub fn new(cx: &mut Context, scale: GridScale) -> Handle<'_, Self> {
        let axe_values = scale.compile();
        Self {
            axe_values,
            orientation: Orientation::Horizontal,
        }
        .build(cx, |_| {})
    }

    /// Helper to create a simple bounding box grid (lines at 0.0 and 1.0).
    /// You have to call this on both orientation to make the complete box
    pub fn border(cx: &mut Context) -> Handle<'_, Self> {
        Self {
            axe_values: GridScale::lines(vec![(0., 0.), (1.0, 1.0)]).compile(),
            orientation: Orientation::Horizontal,
        }
        .build(cx, |_| {})
    }
}
