use crate::dev_prelude::*;

pub struct Panel;

impl Panel {
    /// A panel titled `title`.
    pub fn new(
        cx: &mut Context,
        title: impl Into<String>,
        content: impl FnOnce(&mut Context),
    ) -> Handle<'_, Self> {
        let title = title.into().to_uppercase();

        Self.build(cx, move |cx| {
            Label::new(cx, title).class("panel-title").hoverable(false);
            HStack::new(cx, content).class("panel-body");
        })
    }

    /// A panel without title.
    pub fn bare(cx: &mut Context, content: impl FnOnce(&mut Context)) -> Handle<'_, Self> {
        Self.build(cx, move |cx| {
            HStack::new(cx, content).class("panel-body");
        })
    }
}

impl View for Panel {
    fn element(&self) -> Option<&'static str> {
        Some("panel")
    }
}
