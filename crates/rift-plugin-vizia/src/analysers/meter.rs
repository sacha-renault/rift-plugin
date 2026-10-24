use crate::dev_prelude::*;
use vizia::vg::{RRect, Rect};

struct DrawConfig {
    segment: f32,
    gap: f32,
    warn_from: f32,
    hot_from: f32,
}

impl DrawConfig {
    const SEGMENT: f32 = 3.0;
    const GAP: f32 = 1.5;
    const WARN_FROM: f32 = 0.7;
    const HOT_FROM: f32 = 0.92;
}

impl Default for DrawConfig {
    fn default() -> Self {
        Self {
            segment: Self::SEGMENT,
            gap: Self::GAP,
            warn_from: Self::WARN_FROM,
            hot_from: Self::HOT_FROM,
        }
    }
}

/// A segmented level meter
pub struct MeterUI {
    level: Signal<f32>,
    peak: Signal<f32>,
    vertical: Signal<bool>,
    config: DrawConfig,
}

impl MeterUI {
    /// A meter showing `level`.
    pub fn new(cx: &mut Context, level: Signal<f32>) -> Handle<'_, Self> {
        Self::with_peak(cx, level, Signal::new(0.0))
    }

    /// A meter showing `level`, with a marker at `peak`.
    pub fn with_peak(cx: &mut Context, level: Signal<f32>, peak: Signal<f32>) -> Handle<'_, Self> {
        Self {
            level,
            peak,
            config: DrawConfig::default(),
            vertical: Signal::new(true),
        }
        .build(cx, |_| {})
        .bind(level, |mut handle| handle.needs_redraw())
        .bind(peak, |mut handle| handle.needs_redraw())
        .role(Role::Meter)
        .numeric_value(level.map(|v| (*v as f64 * 100.0).round()))
    }
}

impl View for MeterUI {
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
        let pitch = (self.config.segment + self.config.gap) * scale;
        let count = ((length + self.config.gap * scale) / pitch)
            .floor()
            .max(1.0) as usize;
        // Spread the leftover space so both ends are flush with the bounds.
        let pitch = (length + self.config.gap * scale) / count as f32;
        let segment = pitch - self.config.gap * scale;

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
            let lit_color = if t < self.config.warn_from {
                slots.accent
            } else if t < self.config.hot_from {
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

#[modifiers(for Handle<'_, MeterUI>)]
pub trait MeterUIModifers {
    /// Change direction to horizontal
    #[concrete]
    fn horizontal(self) -> Self {
        self.modify(|meter| meter.vertical.set(false))
    }

    /// Change the gap size
    #[concrete]
    fn gap(self, gap: f32) -> Self {
        let mut ui = self.modify(|meter| meter.config.gap = gap);
        ui.needs_redraw();
        ui
    }

    /// Change the segment size
    #[concrete]
    fn segment(self, segment: f32) -> Self {
        let mut ui = self.modify(|meter| meter.config.segment = segment);
        ui.needs_redraw();
        ui
    }

    /// Change the segment size
    #[concrete]
    fn hot_from(self, hot_from: f32) -> Self {
        // We don't ensure here that hot_from > warnfrom. Otherwise, if user
        // chain hot_from(<value>).warn_from(<other_value>) with a <value>
        // smaller than the previous_warn from, it would clamp it even if the
        // pair (<value>, <other_value>) is valid ...
        let mut ui = self.modify(|meter| meter.config.hot_from = hot_from);
        ui.needs_redraw();
        ui
    }

    /// Change the segment size
    #[concrete]
    fn warn_from(self, warn_from: f32) -> Self {
        // Same comment than in hot from be reversed.
        let mut ui = self.modify(|meter| meter.config.warn_from = warn_from);
        ui.needs_redraw();
        ui
    }
}
