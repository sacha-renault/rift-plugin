use super::gui_prelude::*;

#[derive(Lens)]
pub struct CustomPopup {
    /// Whether the popup is currently visible (used for toggling).
    pub is_open: PopupData,

    /// Where to position the dropdown relative to the trigger element.
    pub placement: Placement,

    /// Whether to automatically adjust position if another UI element obstructs it.
    pub should_reposition: bool,

    /// Whether to render an arrow pointing to the trigger element.
    pub show_arrow: bool,

    /// The size (width/height) of the arrow graphic.
    pub arrow_size: Length,
}

impl Default for CustomPopup {
    fn default() -> Self {
        Self {
            is_open: PopupData { is_open: false },
            placement: Placement::Bottom,
            should_reposition: true,
            show_arrow: false,
            arrow_size: Length::Value(LengthValue::Px(0.)),
        }
    }
}

impl View for CustomPopup {
    fn element(&self) -> Option<&'static str> {
        Some("popup")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        log::info!("Popup event");
        self.is_open.event(cx, event);
    }
}

impl CustomPopup {
    pub fn new<F>(cx: &mut Context, content: F) -> Handle<'_, Self>
    where
        F: 'static + Fn(&mut Context),
    {
        Self::default().build(cx, |cx| {
            Binding::new(cx, CustomPopup::is_open, move |cx, is_open| {
                if is_open.get(cx).into() {
                    Popup::new(cx, |cx| content(cx))
                        // .on_blur(|cx| on_blur(cx))
                        .placement(CustomPopup::placement)
                        .show_arrow(CustomPopup::show_arrow)
                        .arrow_size(CustomPopup::arrow_size)
                        .should_reposition(CustomPopup::should_reposition);
                }
            })
        })
    }
}

/// Trait providing a way to add any [`DestructThenBuildView`] into a [`Popup`].
///
/// # Important:
/// Calling this will override the [`ActionModifiers::on_mouse_down`] of the target element.
/// To safely use this on an element that already has click handlers, wrap it in a container
/// like a [`VStack`] or [`HStack`].
pub trait PopupExt {
    fn popup_base<F>(&mut self, content: F) -> Entity
    where
        F: 'static + Fn(&mut Context);

    /// Add a popup to the receiver. The popup will be opened with a btn click.
    ///
    /// # Arguments
    /// * `into_view` - The builder that will be builded into the popup view.
    /// * `trigger_btn` - The mouse button event that will trigger the dropdown to open.
    fn popup_on_click<F>(self, content: F, trigger_btn: MouseButton) -> Self
    where
        F: 'static + Fn(&mut Context);

    /// Add a popup to the receiver. The popup will be opened with a btn click.
    ///
    /// # Arguments
    /// * `into_view` - The builder that will be builded into the popup view.
    /// * `lens` - The lens that will be watched to know when to open the popup.
    fn popup_on<F, L>(self, content: F, lens: L) -> Self
    where
        F: 'static + Fn(&mut Context),
        L: Lens<Target = bool>;

    /// # Warning
    /// This method is not ready for use:
    /// - Mouse cannot reach the popup before it disappears
    /// - Popup captures clicks unexpectedly
    ///
    /// # Notes:
    /// it's not really deprecated because it has never been ready but people
    /// will read doc when seeing deprecated and that's what i want
    #[deprecated = "UNSTABLE"]
    fn popup_on_hover<F>(self, content: F) -> Self
    where
        F: 'static + Fn(&mut Context);
}

impl<V> PopupExt for Handle<'_, V>
where
    V: View,
{
    fn popup_base<F>(&mut self, content: F) -> Entity
    where
        F: 'static + Fn(&mut Context),
    {
        let trigger_entity = self.entity();
        self.context().with_current(trigger_entity, |cx| {
            CustomPopup::new(cx, move |cx| {
                content(cx);
            })
            // .height(Pixels(0.))
            // .width(Pixels(0.))
            .entity()
            // Build the view with 0 width/height initially (hidden)
        })
    }

    fn popup_on_click<F>(mut self, content: F, trigger_btn: MouseButton) -> Self
    where
        F: 'static + Fn(&mut Context),
    {
        let entity = self.popup_base(content);

        self.on_mouse_down(move |cx, mb| {
            if mb == trigger_btn {
                // Emit the open event to the unique popup entity
                cx.emit_to(entity, PopupEvent::Open);
            }
        })
    }

    fn popup_on<F, L>(mut self, content: F, lens: L) -> Self
    where
        F: 'static + Fn(&mut Context),
        L: Lens<Target = bool>,
    {
        let entity = self.popup_base(content);

        Binding::new(self.context(), lens, move |cx, lens| {
            if lens.get(cx) {
                cx.emit_to(entity, PopupEvent::Open);
            } else {
                cx.emit_to(entity, PopupEvent::Close);
            }
        });

        self
    }

    fn popup_on_hover<F>(mut self, content: F) -> Self
    where
        F: 'static + Fn(&mut Context),
    {
        let entity = self.popup_base(content);

        self.on_over(move |cx| cx.emit_to(entity, PopupEvent::Open))
            .on_over_out(move |cx| cx.emit_to(entity, PopupEvent::Close))
    }
}
