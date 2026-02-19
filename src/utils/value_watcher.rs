use paste::paste;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

macro_rules! define_value_watcher {
    ($inner_type:ident) => {
        paste! {
            /// # Threading model
            ///
            /// This type assumes a **single writer** and **single reader** on different threads.
            /// Concurrent writes or concurrent reads are not protected against and may cause
            /// values to be read twice or missed. In CLAP context this means:
            /// - Writer: audio thread
            /// - Reader: main thread
            ///
            /// Using this type outside of this assumption is undefined behavior.
            pub struct [<ValueWatcher $inner_type:camel>] {
                has_changed: AtomicBool,
                value: [<Atomic $inner_type:camel>],
            }

            impl [<ValueWatcher $inner_type:camel>] {
                pub fn new(value: $inner_type) -> Self {
                    Self {
                        has_changed: AtomicBool::new(false),
                        value: <[<Atomic $inner_type:camel>]>::new(value),
                    }
                }

                /// Same as [`Self::new`] but has_changed starts at true
                pub fn new_changed(value: $inner_type)-> Self {
                    Self {
                        has_changed: AtomicBool::new(true),
                        value: <[<Atomic $inner_type:camel>]>::new(value),
                    }
                }

                /// This function only cares about the data has changed ... not about the data is
                /// ready to being read !
                ///
                /// DO NOT USE THIS LIKE:
                /// ```compile_fail
                /// if watcher.take_changed() {
                ///     let value = watcher.value(); // WRONG!!
                /// }
                /// ```
                ///
                /// To know if there is change + get value, use
                /// [`Self::take_changed_value`]
                /// ```
                /// if let Some(new_value) = watcher.take_changed_value() {
                ///     // Do stufs ...
                /// }
                /// ```
                #[inline]
                pub fn take_changed(&self) -> bool {
                    self.has_changed.swap(false, Ordering::Relaxed)
                }

                /// To know if value has changed AND get the value
                /// conditions like
                /// ```ignore
                /// if watcher.take_changed_value().is_some() { ... }
                /// ```
                /// will work but isn't recommended.
                /// Use [`Self::take_changed`]
                /// instead
                #[inline]
                pub fn take_changed_value(&self) -> Option<$inner_type> {
                    if self.has_changed.swap(false, Ordering::Acquire) {
                        Some(self.value.load(Ordering::Relaxed))
                    } else {
                        None
                    }
                }

                /// Set a new value and set true in changed
                ///
                /// If the value is the same, it will still trigger has_changed.
                #[inline]
                pub fn set_new_value(&self, value: $inner_type) {
                    self.value.store(value, Ordering::Relaxed);
                    self.has_changed.store(true, Ordering::Release);
                }

                /// Set a value into the watcher, if the value is the same
                /// It wont trigger a new has_changed = true
                #[inline]
                pub fn set_value(&self, value: $inner_type) {
                    let old_value = self.value.swap(value, Ordering::Relaxed);
                    if old_value != value {
                        self.has_changed.store(true, Ordering::Release);
                    }
                }

                /// Get the current value.
                ///
                /// This function SHOULDN'T be used this way:
                /// ```compile_fail
                /// if watcher.take_changed() {
                ///     let value = watcher.value(); // WRONG!!
                /// }
                /// ```
                ///
                /// If querying value only if changed, use [`Self::take_changed_value`]:
                /// ```
                /// if let Some(new_value) = watcher.take_changed_value() {
                ///     // Do stufs ...
                /// }
                /// ```
                #[inline]
                pub fn value(&self) -> $inner_type {
                    self.value.load(Ordering::Relaxed)
                }
            }
        }
    };
}

define_value_watcher! {u32}
