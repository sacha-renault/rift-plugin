//! this module contains generic extensions for Handle<'_, V>

use vizia::prelude::*;

pub struct FView;

impl View for FView {}

pub trait ViewApplyModifiers<'a>: Sized {
    /// Applies an optional function to a Handle, rebranding it as an FView.
    ///
    /// Looses the possibility to apply any method for Handle<'_, V> where V: View
    /// but allow basic Layout modifiers etc ...
    fn maybe_apply_modifiers<F>(self, func: Option<F>) -> Handle<'a, FView>
    where
        F: Fn(Handle<'a, FView>) -> Handle<'a, FView>;
}

impl<'a, T> ViewApplyModifiers<'a> for Handle<'a, T>
where
    T: View,
{
    fn maybe_apply_modifiers<F>(self, func: Option<F>) -> Handle<'a, FView>
    where
        F: Fn(Handle<'a, FView>) -> Handle<'a, FView>,
    {
        // SAFETY: Handle<T> and Handle<FView> are identical in layout,
        // PhantomData<T> is a ZST. We're just rebranding the type tag.
        let handle: Handle<'a, FView> = unsafe { std::mem::transmute(self) };

        if let Some(f) = func {
            f(handle)
        } else {
            handle
        }
    }
}

pub trait RedrawOnExt: Sized {
    /// Binds a [`Lens`] to this Handle to trigger redraws when the lens output changes.
    ///
    /// The bound Lens must target a `u64`. This method attaches a binding that calls `needs_redraw`
    /// on the current entity whenever the lens fires, ensuring the UI updates reactively.
    fn redraw_on<L>(self, lens: L) -> Self
    where
        L: Lens<Target = u64>;
}

impl<V> RedrawOnExt for Handle<'_, V>
where
    V: View,
{
    fn redraw_on<L>(mut self, lens: L) -> Self
    where
        L: Lens<Target = u64>,
    {
        let entity = self.entity();
        Binding::new(self.context(), lens, move |cx, _| {
            cx.needs_redraw(entity);
        });
        self
    }
}
