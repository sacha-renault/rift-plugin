use std::any::Any;
use std::mem::{ManuallyDrop, MaybeUninit};
use std::ptr;

/// The only point of this macro is if ALIGN change it is changed everywhere.
macro_rules! define_storage {
    (CAP = $cap:literal, ALIGN = $align:literal) => {
        const CAP: usize = $cap;
        const ALIGN: usize = $align;

        #[repr(C, align($align))]
        struct Storage(MaybeUninit<[u8; $cap]>);

        const _: () = assert!(align_of::<Storage>() == ALIGN, "Not aligned");
    };
}

define_storage! {CAP = 128, ALIGN = 16}

pub struct BackgroundTask {
    storage: Storage,
    call: unsafe fn(*mut u8) -> Box<dyn Any>,
    drop: unsafe fn(*mut u8),
}

unsafe fn call_impl<F, V>(p: *mut u8) -> Box<dyn Any>
where
    V: 'static,
    F: FnOnce() -> Box<V>,
{
    // Move the closure out of the buffer, then consume it.
    let f = unsafe { ptr::read(p as *mut F) };
    f()
}

unsafe fn drop_impl<F>(p: *mut u8) {
    unsafe { ptr::drop_in_place(p as *mut F) }
}

impl BackgroundTask {
    pub fn new<F, V>(f: F) -> Self
    where
        V: Send + 'static,
        F: FnOnce() -> Box<V> + Send + 'static,
    {
        // Compile-time check
        const {
            assert!(size_of::<F>() <= CAP, "closure captures too much");
            assert!(align_of::<F>() <= ALIGN, "closure over-aligned");
        }

        let mut storage = Storage(MaybeUninit::uninit());
        unsafe { ptr::write(storage.0.as_mut_ptr() as *mut F, f) };

        Self {
            storage,
            call: call_impl::<F, V>,
            drop: drop_impl::<F>,
        }
    }

    pub fn run(self) -> Task {
        let call = self.call;
        // Prevent Drop from also dropping the closure we're about to consume.
        let mut this = ManuallyDrop::new(self);
        let v = unsafe { call(this.storage.0.as_mut_ptr() as *mut u8) };
        Task(v)
    }
}

impl Drop for BackgroundTask {
    fn drop(&mut self) {
        // Task was never run: drop the captured data.
        unsafe { (self.drop)(self.storage.0.as_mut_ptr() as *mut u8) }
    }
}

pub struct Task(Box<dyn Any>);

impl Task {
    pub fn new<V: Send + 'static>(v: V) -> Self {
        Self(Box::new(v))
    }

    pub fn inner(self) -> Box<dyn Any> {
        self.0
    }
}

// SAFETY: `Task` is only ever built from a `V: Send` (`BackgroundTask::new` / `Task::new`),
// so moving it to another thread is sound even though `Box<dyn Any>` isn't `Send`.
unsafe impl Send for Task {}
