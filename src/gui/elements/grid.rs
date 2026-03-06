use vizia::vg;

use super::gui_prelude::*;

pub enum GridScale {
    Linear {
        start: f32,
        end: f32,
        count: usize,
    },
    Logarithmic {
        start: f32,
        end: f32,
        base: f32,
        sub_ticks: usize,
    },
    Lines(Vec<(f32, f32)>),
}

impl GridScale {
    pub fn linear(start: f32, end: f32, count: usize) -> Self {
        Self::Linear { start, end, count }
    }

    pub fn logarithmic(start: f32, end: f32, base: f32, sub_ticks: usize) -> Self {
        Self::Logarithmic {
            start,
            end,
            base,
            sub_ticks,
        }
    }

    pub fn line(normalized: f32, value: f32) -> Self {
        Self::Lines(vec![(normalized, value)])
    }

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
        let values = (0..count)
            .map(|i| {
                let t = i as f32 / (count - 1) as f32;
                GridValue {
                    normalized: t,
                    value: start + t * range,
                }
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
struct GridValue {
    normalized: f32,

    // ALLOW:
    // We might later plot the value
    // So we keep this here for now
    #[allow(unused)]
    value: f32,
}

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

        // The draw loop is now incredibly clean and fast. No math, just mapping.
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
    pub fn new(cx: &mut Context, scale: GridScale) -> Handle<'_, Self> {
        let axe_values = scale.compile();
        Self {
            axe_values,
            orientation: Orientation::Horizontal,
        }
        .build(cx, |_| {})
    }
}
