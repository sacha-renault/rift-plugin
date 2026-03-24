use vizia::prelude::*;

#[derive(Lens)]
pub struct ActiveElementData {
    pub is_dragging: bool,
    pub is_over: bool,
    cursor: CursorIcon,
}

impl ActiveElementData {
    pub fn new(cursor: CursorIcon) -> Self {
        Self {
            is_dragging: false,
            is_over: false,
            cursor,
        }
    }
}

impl Default for ActiveElementData {
    fn default() -> Self {
        Self::new(CursorIcon::Default)
    }
}

pub enum ActiveElementEvent {
    StartDragging,
    EndDragging,
    StartHover,
    EndHover,
}

impl Model for ActiveElementData {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|event, meta| {
            match event {
                ActiveElementEvent::StartDragging => {
                    self.is_dragging = true;
                    cx.lock_cursor_icon();
                    cx.emit(WindowEvent::SetCursor(self.cursor));
                }
                ActiveElementEvent::EndDragging => {
                    self.is_dragging = false;
                    cx.unlock_cursor_icon();
                }
                ActiveElementEvent::StartHover => {
                    self.is_over = true;
                    if !cx.is_cursor_icon_locked() {
                        cx.emit(WindowEvent::SetCursor(self.cursor));
                    }
                }
                ActiveElementEvent::EndHover => {
                    self.is_over = false;
                    if !cx.is_cursor_icon_locked() {
                        cx.emit(WindowEvent::SetCursor(CursorIcon::Default));
                    }
                }
            };

            meta.consume();
        });
    }
}
