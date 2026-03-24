use std::marker::PhantomData;

use super::gui_prelude::*;

// #[derive(HandleExtension)]
pub struct PositionIndicator<L>
where
    L: Lens<Target = f32>,
{
    _p: PhantomData<L>,
}

impl<L> PositionIndicator<L>
where
    L: Lens<Target = f32>,
{
    pub fn new(cx: &mut Context, position: L) -> Handle<'_, Self> {
        Self { _p: PhantomData }
            .build(cx, |cx| {
                Element::new(cx)
                    .position_type(PositionType::Absolute)
                    .height(Percentage(100.))
                    .left(position.map(|x| Percentage(*x * 100.)))
                    .class("position-indicator");
            })
            .pointer_events(false)
            .navigable(false)
            .focusable(false)
    }
}

impl<L> View for PositionIndicator<L> where L: Lens<Target = f32> {}
