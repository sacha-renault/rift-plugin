use std::{mem::MaybeUninit, sync::OnceLock};

use crossbeam_channel::{Sender, bounded};

const SLOT_SIZE: usize = 64;
const SLOT_ALIGN: usize = 16;

#[repr(align(16))]
struct Storage([MaybeUninit<u8>; SLOT_SIZE]);

/// Any `Send` value, stored inline, with its destructor remembered.
pub struct Garbage {
    data: Storage,
    drop_fn: unsafe fn(*mut u8),
}

impl Garbage {
    fn new<T: Send + 'static>(value: T) -> Self {
        // Compile-time errors if the type doesn't fit
        const { assert!(size_of::<T>() <= SLOT_SIZE, "type too big for Garbage slot") };
        const {
            assert!(
                align_of::<T>() <= SLOT_ALIGN,
                "type over-aligned for Garbage slot"
            )
        };

        unsafe fn drop_it<T>(p: *mut u8) {
            unsafe { std::ptr::drop_in_place(p as *mut T) }
        }

        let mut data = Storage([MaybeUninit::uninit(); SLOT_SIZE]);
        unsafe { std::ptr::write(data.0.as_mut_ptr() as *mut T, value) };
        Garbage {
            data,
            drop_fn: drop_it::<T>,
        }
    }
}

// Only ever built from `T: Send`.
unsafe impl Send for Garbage {}

impl Drop for Garbage {
    fn drop(&mut self) {
        unsafe { (self.drop_fn)(self.data.0.as_mut_ptr() as *mut u8) }
    }
}

static GRAVEYARD: OnceLock<Sender<Garbage>> = OnceLock::new();

pub fn init_graveyard(capacity: usize) {
    GRAVEYARD.get_or_init(|| {
        let (sender, rx) = bounded(capacity);

        std::thread::Builder::new()
            .name("graveyard".into())
            .spawn(move || {
                for g in rx {
                    std::mem::drop(g);
                }
            })
            .expect("failed to spawn graveyard thread");

        sender
    });
}

/// RT-safe, works for ANY `Send + 'static` type that fits the slot.
pub fn drop_rt_safe<T: Send + 'static>(value: T) {
    if !std::mem::needs_drop::<T>() {
        return;
    }

    let g = Garbage::new(value);
    match GRAVEYARD.get() {
        Some(queue) => {
            if let Err(g) = queue.try_send(g) {
                std::mem::forget(g);
            }
        }
        None => std::mem::forget(g),
    }
}

/// Replaces the value of `slot` by `new`, then
/// send the old value of `slot` to be deallocated
/// in an other thread so there is no dealloc on the
/// audio thread. This is useless for a type that doesn't
/// deallocate.
pub fn swap_in<T: Send + 'static>(slot: &mut T, mut new: T) {
    std::mem::swap(slot, &mut new);
    drop_rt_safe(new);
}
