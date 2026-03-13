use super::gui_prelude::*;

/// Trait providing a way to add any [`DestructThenBuildView`] into a [`Popup`].
///
/// # Important:
/// Calling this will override the [`ActionModifiers::on_mouse_down`] of the target element.
/// To safely use this on an element that already has click handlers, wrap it in a container
/// like a [`VStack`] or [`HStack`].
pub trait PopupExt {
    fn popup_base<D>(&mut self, into_view: D) -> Entity
    where
        D: DestructThenBuildView;

    /// Add a popup to the receiver. The popup will be opened with a btn click.
    ///
    /// # Arguments
    /// * `into_view` - The builder that will be builded into the popup view.
    /// * `trigger_btn` - The mouse button event that will trigger the dropdown to open.
    fn popup_on_click<D>(self, into_view: D, trigger_btn: MouseButton) -> Self
    where
        D: DestructThenBuildView;

    /// Add a popup to the receiver. The popup will be opened with a btn click.
    ///
    /// # Arguments
    /// * `into_view` - The builder that will be builded into the popup view.
    /// * `lens` - The lens that will be watched to know when to open the popup.
    fn popup_on<D, L>(self, into_view: D, lens: L) -> Self
    where
        D: DestructThenBuildView,
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
    fn popup_on_hover<D>(self, into_view: D) -> Self
    where
        D: DestructThenBuildView;
}

impl<V> PopupExt for Handle<'_, V>
where
    V: View,
{
    fn popup_base<D>(&mut self, into_view: D) -> Entity
    where
        D: DestructThenBuildView,
    {
        let trigger_entity = self.entity();
        self.context().with_current(trigger_entity, |cx| {
            // Build the view with 0 width/height initially (hidden)
            into_view
                .build_view(cx)
                .height(Pixels(0.))
                .width(Pixels(0.))
                .entity()
        })
    }

    fn popup_on_click<D>(mut self, into_view: D, trigger_btn: MouseButton) -> Self
    where
        D: DestructThenBuildView,
    {
        let entity = self.popup_base(into_view);

        self = self.on_mouse_down(move |cx, mb| {
            if mb == trigger_btn {
                // Emit the open event to the unique popup entity
                cx.emit_to(entity, PopupEvent::Open);
            }
        });

        self
    }

    fn popup_on<D, L>(mut self, into_view: D, lens: L) -> Self
    where
        D: DestructThenBuildView,
        L: Lens<Target = bool>,
    {
        let entity = self.popup_base(into_view);

        Binding::new(self.context(), lens, move |cx, lens| {
            if lens.get(cx) {
                cx.emit_to(entity, PopupEvent::Open);
            } else {
                cx.emit_to(entity, PopupEvent::Close);
            }
        });

        self
    }

    fn popup_on_hover<D>(mut self, into_view: D) -> Self
    where
        D: DestructThenBuildView,
    {
        let entity = self.popup_base(into_view);

        self = self
            .on_hover(move |cx| cx.emit_to(entity, PopupEvent::Open))
            .on_hover_out(move |cx| {
                if !cx.is_over() {
                    cx.emit_to(entity, PopupEvent::Close);
                }
            });

        self
    }
}
