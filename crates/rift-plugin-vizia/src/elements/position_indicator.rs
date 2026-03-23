use vizia::vg;

use super::gui_prelude::*;

// #[derive(HandleExtension)]
pub struct PositionIndicator<L>
where
    L: Lens<Target = f32>,
{
    position: L,
}

impl<L> PositionIndicator<L>
where
    L: Lens<Target = f32>,
{
    pub fn new(cx: &mut Context, position: L) -> Handle<'_, Self> {
        Self { position }
            .build(cx, |_| {})
            .pointer_events(false)
            .navigable(false)
            .focusable(false)
    }
}

impl<L> View for PositionIndicator<L>
where
    L: Lens<Target = f32>,
{
    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let vt = ViewportTransform::new(cx.bounds());
        let pos = self.position.get(cx);

        let start = vt.transform(pos, 0.);
        let end = vt.transform(pos, 1.);

        let mut paint = vg::Paint::default();
        paint
            .set_color(cx.font_color())
            .set_stroke_width(cx.border_width());

        canvas.draw_line(start, end, &paint);
    }
}
