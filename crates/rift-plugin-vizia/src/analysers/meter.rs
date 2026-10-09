use crate::dev_prelude::*;
use vizia::vg::{RRect, Rect};

const SEGMENT: f32 = 3.0;
const GAP: f32 = 1.5;
const WARN_FROM: f32 = 0.7;
const HOT_FROM: f32 = 0.92;

/// A segmented level meter
pub struct Meter {
    level: Signal<f32>,
    peak: Signal<f32>,
    vertical: Signal<bool>,
}

impl Meter {
    /// A meter showing `level`.
    pub fn new(cx: &mut Context, level: Signal<f32>) -> Handle<'_, Self> {
        Self::with_peak(cx, level, Signal::new(0.0))
    }

    /// A meter showing `level`, with a marker at `peak`.
    pub fn with_peak(cx: &mut Context, level: Signal<f32>, peak: Signal<f32>) -> Handle<'_, Self> {
        Self {
            level,
            peak,
            vertical: Signal::new(true),
        }
        .build(cx, |_| {})
        .bind(level, |mut handle| handle.needs_redraw())
        .bind(peak, |mut handle| handle.needs_redraw())
        .role(Role::Meter)
        .numeric_value(level.map(|v| (*v as f64 * 100.0).round()))
    }
}

impl View for Meter {
    fn element(&self) -> Option<&'static str> {
        Some("meter")
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let bounds = cx.bounds();
        if bounds.w < 1.0 || bounds.h < 1.0 {
            return;
        }

        let slots = Slots::read(cx);
        let scale = cx.scale_factor();
        let vertical = self.vertical.get_untracked();

        let (length, thickness) = if vertical {
            (bounds.h, bounds.w)
        } else {
            (bounds.w, bounds.h)
        };
        let pitch = (SEGMENT + GAP) * scale;
        let count = ((length + GAP * scale) / pitch).floor().max(1.0) as usize;
        // Spread the leftover space so both ends are flush with the bounds.
        let pitch = (length + GAP * scale) / count as f32;
        let segment = pitch - GAP * scale;

        let level = self.level.get_untracked().clamp(0.0, 1.0);
        let peak = self.peak.get_untracked().clamp(0.0, 1.0);
        let peak_segment = ((peak * count as f32).ceil() as usize).clamp(1, count) - 1;

        let warn = if slots.glow.a() > 0 {
            slots.glow
        } else {
            Color::rgb(255, 200, 87)
        };

        for index in 0..count {
            let t = (index as f32 + 0.5) / count as f32;
            let lit_color = if t < WARN_FROM {
                slots.accent
            } else if t < HOT_FROM {
                warn
            } else {
                slots.accent_end
            };

            let lit = t <= level + 0.5 / count as f32 && level > 0.0;
            let is_peak = peak > 0.0 && index == peak_segment;
            let color = if lit {
                lit_color
            } else if is_peak {
                mix(slots.track, lit_color, 0.85)
            } else {
                slots.track
            };

            // Segments are laid out from the start (bottom or left).
            let offset = index as f32 * pitch;
            let rect = if vertical {
                Rect::from_xywh(
                    bounds.x,
                    bounds.bottom() - offset - segment,
                    thickness,
                    segment,
                )
            } else {
                Rect::from_xywh(bounds.x + offset, bounds.y, segment, thickness)
            };

            let radius = (thickness.min(segment) * 0.3).max(0.5);
            let rrect = RRect::new_rect_xy(rect, radius, radius);
            canvas.draw_rrect(rrect, &fill_paint(color));
        }
    }
}

#[modifiers(for Handle<'_, Meter>)]
pub trait MeterModifers {
    #[concrete]
    /// Change direction to horizontal
    fn horizontal(self) -> Self {
        self.modify(|meter| meter.vertical.set(false))
    }
}
