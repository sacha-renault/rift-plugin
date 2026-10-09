use crate::dev_prelude::*;

#[modifiers(for Button)]
pub trait ButtonExt {
    #[concrete]
    fn icon<T1>(cx: &mut Context, text: T1, icon: impl Res<String>) -> Handle<'_, Button>
    where
        T1: Res<String> + Clone,
        T1: 'static,
    {
        Button::new(cx, move |cx| {
            ZStack::new(cx, |cx| {
                Svg::new(cx, icon).hoverable(false);
            })
            .hoverable(false)
            .class("icon");

            Label::new(cx, text)
        })
    }

    #[concrete]
    fn label<T>(cx: &mut Context, text: T) -> Handle<'_, Button>
    where
        T: Res<String> + Clone,
        T: 'static,
    {
        Button::new(cx, move |cx| Label::new(cx, text).hoverable(false))
    }
}

#[modifiers(for Handle<'_, Button>)]
pub trait ButtonModifiers2 {
    #[concrete]
    fn reversed(self) -> Self {
        self.class("reversed")
    }

    #[concrete]
    fn small(self) -> Self {
        self.class("small").toggle_class("large", false)
    }

    #[concrete]
    fn large(self) -> Self {
        self.class("large").toggle_class("small", false)
    }
}
