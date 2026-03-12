use std::sync::Arc;

use super::gui_prelude::*;

pub trait AddDropdown {
    /// Add a dropdown to an arbitrary element
    ///
    /// # Important:
    /// this will override the [`ActionModifiers::on_mouse_down`], to use it,
    /// either ensure the view has no action callback on this or wrap it into a view
    /// like a [`VStack`] or [`HStack`]. (todo()! might need to add `Div` like element?)
    fn add_dropdown(self, trigger_btn: MouseButton, items: Vec<DropdownItem>) -> Self;
}

impl<V> AddDropdown for Handle<'_, V>
where
    V: View,
{
    fn add_dropdown(mut self, trigger_btn: MouseButton, items: Vec<DropdownItem>) -> Self {
        let entity = DropdownData::default()
            .build(self.context(), |cx| {
                Binding::new(cx, DropdownData::is_open, move |cx, is_open| {
                    if is_open.get(cx).into() {
                        Popup::new(cx, |cx| build_popup_content(cx, &items))
                            .on_blur(|cx| cx.emit(PopupEvent::Close))
                            .placement(DropdownData::placement)
                            .show_arrow(false)
                            .should_reposition(DropdownData::should_reposition)
                            .class("dropdown-popup");
                    }
                })
            })
            .width(Pixels(0.))
            .height(Pixels(0.))
            .entity();

        self = self.on_mouse_down(move |cx, mb| {
            if mb == trigger_btn {
                cx.emit_to(entity, PopupEvent::Open);
            }
        });

        self
    }
}

fn build_popup_content(cx: &mut Context, items: &Vec<DropdownItem>) {
    for DropdownItem {
        name,
        action,
        class,
    } in items.iter()
    {
        // Clone action so closure can capture it
        let action = action.clone();
        let label = Label::new(cx, name).on_mouse_down(move |cx, _| {
            if let Some(action) = action.clone() {
                action(cx)
            }
        });

        if let Some(class) = &class {
            label.class(class.as_ref());
        }
    }
}

#[derive(Lens)]
struct DropdownData {
    pub is_open: PopupData,
    pub placement: Placement,
    pub should_reposition: bool,
}

impl Default for DropdownData {
    fn default() -> Self {
        Self {
            is_open: PopupData::default(),
            placement: Placement::Left,
            should_reposition: true,
        }
    }
}

impl View for DropdownData {
    fn element(&self) -> Option<&'static str> {
        Some("dropdown")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        self.is_open.event(cx, event);
    }
}

#[derive(Clone)]
pub struct DropdownItem {
    name: String,
    action: Option<Arc<dyn Fn(&mut EventContext) + Send + Sync>>,
    class: Option<String>,
    /*Add here some enum for type */
}

impl View for DropdownItem {}

impl DropdownItem {
    pub fn new<F>(name: String, func: F) -> Self
    where
        F: 'static + Fn(&mut EventContext) + Send + Sync,
    {
        Self {
            name,
            action: Some(Arc::new(func)),
            class: None,
        }
    }
}
