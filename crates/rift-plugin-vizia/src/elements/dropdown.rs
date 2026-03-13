use std::sync::Arc;

use super::gui_prelude::*;

/// Trait providing a way to add a [`DropdownStyled`] to any arbitrary element.
///
/// # Important:
/// Calling this will override the [`ActionModifiers::on_mouse_down`] of the target element.
/// To safely use this on an element that already has click handlers, wrap it in a container
/// like a [`VStack`] or [`HStack`].
pub trait AddDropdown {
    /// Add a dropdown to the receiver.
    ///
    /// # Arguments
    /// * `trigger_btn` - The mouse button event that will trigger the dropdown to open.
    /// * `dropdown` - The configuration instance for the dropdown content and behavior.
    ///
    /// # Returns
    /// The modified handle with the dropdown attached.
    fn add_dropdown(self, trigger_btn: MouseButton, dropdown: DropdownStyled) -> Self;
}

impl<V> AddDropdown for Handle<'_, V>
where
    V: View,
{
    fn add_dropdown(mut self, trigger_btn: MouseButton, dropdown: DropdownStyled) -> Self {
        let trigger_entity = self.entity();
        let entity = self.context().with_current(trigger_entity, |cx| {
            // Build the view with 0 width/height initially (hidden)
            dropdown
                .build_view(cx)
                .height(Pixels(0.))
                .width(Pixels(0.))
                .entity()
        });

        self = self.on_mouse_down(move |cx, mb| {
            if mb == trigger_btn {
                // Emit the open event to the unique popup entity
                cx.emit_to(entity, PopupEvent::Open);
            }
        });

        self
    }
}

/// Builds the content for a dropdown.
/// Iterates through items and creates labels with attached actions.
fn build_popup_content(cx: &mut Context, items: &Vec<DropdownItem>) {
    for DropdownItem {
        name,
        action,
        class_name: class,
    } in items.iter()
    {
        // Clone action so closure can capture it
        let action = action.clone();
        let label = Label::new(cx, name).on_mouse_down(move |cx, _| {
            if let Some(action) = action.clone() {
                action(cx)
            }
            cx.emit(PopupEvent::Close);
        });

        if let Some(class) = &class {
            label.class(class.as_ref());
        }
    }
}

/// Represents a styled dropdown configuration and view.
#[derive(Lens)]
pub struct DropdownStyled {
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

    /// The list of actions/items displayed in the dropdown menu.
    pub items: Vec<DropdownItem>,
}

impl Default for DropdownStyled {
    fn default() -> Self {
        Self {
            is_open: PopupData { is_open: false },
            placement: Placement::Bottom,
            should_reposition: true,
            show_arrow: false,
            arrow_size: Length::Value(LengthValue::Px(0.)),
            items: Vec::new(),
        }
    }
}

impl DropdownStyled {
    /// Sets the placement of the dropdown menu.
    pub fn placement_(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    /// Toggles whether the popup should reposition itself to avoid obstruction.
    pub fn should_reposition_(mut self, should_reposition: bool) -> Self {
        self.should_reposition = should_reposition;
        self
    }

    /// Toggles the visibility of the arrow indicator.
    pub fn show_arrow_(mut self, show_arrow: bool) -> Self {
        self.show_arrow = show_arrow;
        self
    }

    /// Sets the size of the arrow indicator.
    pub fn arrow_size_(mut self, arrow_size: Length) -> Self {
        self.arrow_size = arrow_size;
        self
    }

    /// Sets the size of the arrow indicator.
    pub fn add_item(mut self, item: DropdownItem) -> Self {
        self.items.push(item);
        self
    }
}

impl DestructThenBuildView for DropdownStyled {
    fn build_view(self, cx: &mut Context) -> Handle<'_, impl View> {
        self.build(cx, |cx| {
            Binding::new(cx, DropdownStyled::is_open, move |cx, is_open| {
                if is_open.get(cx).into() {
                    let items = DropdownStyled::items.get(cx);
                    Popup::new(cx, |cx| build_popup_content(cx, &items))
                        .on_blur(|cx| cx.emit(PopupEvent::Close))
                        .placement(DropdownStyled::placement)
                        .show_arrow(DropdownStyled::show_arrow)
                        .arrow_size(DropdownStyled::arrow_size)
                        .should_reposition(DropdownStyled::should_reposition)
                        .class("dropdown-popup");
                }
            })
        })
    }
}

impl View for DropdownStyled {
    fn element(&self) -> Option<&'static str> {
        Some("dropdown")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        self.is_open.event(cx, event);
    }
}

/// Represents a single item within a [`DropdownStyled`].
#[derive(Clone)]
pub struct DropdownItem {
    /// The display text for the item (e.g., "Open Settings").
    name: String,

    /// Optional action to execute when this item is clicked.
    /// If provided, this overrides any default handling for the label.
    action: Option<Arc<dyn Fn(&mut EventContext) + Send + Sync>>,

    /// Optional CSS class to apply styling or specific behaviors via `view_class`.
    class_name: Option<String>,
    /*Add here some enum for type */
}

impl DropdownItem {
    /// Creates a new [`DropdownItem`] with an action.
    pub fn new(name: String) -> Self {
        Self {
            name,
            action: None,
            class_name: None,
        }
    }

    pub fn action<F>(mut self, func: F) -> Self
    where
        F: 'static + Fn(&mut EventContext) + Send + Sync,
    {
        self.action = Some(Arc::new(func));
        self
    }

    pub fn class_name(mut self, class_name: impl ToString) -> Self {
        self.class_name = Some(class_name.to_string());
        self
    }
}
