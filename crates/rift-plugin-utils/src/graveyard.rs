//! RT-safe deferred drop.
//!
//! The audio thread hands the value to a background "graveyard" thread that owns
//! the destructor, so nothing frees on the audio thread. If there's nowhere to
//! put it (no graveyard, or a full queue) we drop it on the caller instead:
//! not RT-safe, but bounded. Only a *missing* graveyard is loud in debug (see
//! [`drop_rt_safe`]); a full queue is silent unless `assert_no_alloc`'s allocator
//! is installed. Call [`init_graveyard`] once at startup.

use std::{mem::MaybeUninit, sync::OnceLock};

use crossbeam_channel::{Sender, bounded};

const SLOT_SIZE: usize = 64;
const SLOT_ALIGN: usize = 16;

#[repr(align(16))]
struct Storage([MaybeUninit<u8>; SLOT_SIZE]);

// `SLOT_ALIGN` and the `repr` above have to agree, or a value `Garbage::new`
// accepts could be written misaligned.
const _: () = assert!(align_of::<Storage>() >= SLOT_ALIGN);

/// Any `Send` value, stored inline, with its destructor remembered.
pub struct Garbage {
    data: Storage,
    drop_fn: unsafe fn(*mut u8),
}

impl Garbage {
    const fn new<T: Send + 'static>(value: T) -> Self {
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

impl Drop for Garbage {
    fn drop(&mut self) {
        unsafe { (self.drop_fn)(self.data.0.as_mut_ptr() as *mut u8) }
    }
}

/// Hands `g` to `queue`, or drops it right here if there's nowhere to put it.
fn dispose(queue: Option<&Sender<Garbage>>, g: Garbage) {
    match queue {
        // On `Full`, the error carries the value back, so dropping the error
        // drops it here.
        Some(queue) => std::mem::drop(queue.try_send(g)),
        None => std::mem::drop(g),
    }
}

static GRAVEYARD: OnceLock<Sender<Garbage>> = OnceLock::new();

/// Spawns the graveyard thread. Call it once at startup; later calls are no-ops
/// (the first `capacity` wins).
///
/// `capacity` is clamped to at least 1
pub fn init_graveyard(capacity: usize) {
    use std::panic::{AssertUnwindSafe, catch_unwind};

    GRAVEYARD.get_or_init(|| {
        let (sender, rx) = bounded(capacity.max(1));

        std::thread::Builder::new()
            .name("graveyard".into())
            .spawn(move || {
                for g in rx {
                    // A panicking destructor must take the process down.
                    let dropped = catch_unwind(AssertUnwindSafe(|| drop(g)));
                    if dropped.is_err() {
                        std::process::abort();
                    }
                }
            })
            .expect("failed to spawn graveyard thread");

        sender
    });
}

/// Drops `value` off the audio thread. RT-safe for any `Send + 'static` type that
/// fits a slot.
///
/// With nowhere to put it (graveyard missing or full), it's dropped on the
/// caller instead, and debug builds panic if [`init_graveyard`] was never called.
///
/// A value that doesn't fit the slot is a compile error:
///
/// ```compile_fail
/// struct TooBig([u8; 65]);
/// impl Drop for TooBig {
///     fn drop(&mut self) {}
/// }
/// rift_plugin_utils::drop_rt_safe(TooBig([0; 65]));
/// ```
///
/// ```compile_fail
/// #[repr(align(32))]
/// struct TooAligned([u8; 8]);
/// impl Drop for TooAligned {
///     fn drop(&mut self) {}
/// }
/// rift_plugin_utils::drop_rt_safe(TooAligned([0; 8]));
/// ```
pub fn drop_rt_safe<T: Send + 'static>(value: T) {
    if !std::mem::needs_drop::<T>() {
        return;
    }

    let g = Garbage::new(value);

    let queue = GRAVEYARD.get();
    debug_assert!(
        queue.is_some(),
        "drop_rt_safe called before `init_graveyard`: the value will be freed on the audio thread"
    );

    dispose(queue, g);
}

/// Replaces the value of `slot` by `new`, then
/// send the old value of `slot` to be deallocated
/// in an other thread so there is no dealloc on the
/// audio thread. This is useless for a type that doesn't
/// deallocate.
///
/// Note: if drop of the old value of slot panics, it takes
/// down the process.
pub fn swap_in<T: Send + 'static>(slot: &mut T, mut new: T) {
    std::mem::swap(slot, &mut new);
    drop_rt_safe(new);
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{
            atomic::{AtomicU32, Ordering},
            mpsc,
        },
        thread::{self, ThreadId},
        time::Duration,
    };

    use crossbeam_channel::bounded;

    use super::*;

    /// Runs `f` on drop.
    struct OnDrop<F: FnOnce() + Send + 'static>(Option<F>);

    impl<F: FnOnce() + Send + 'static> OnDrop<F> {
        fn new(f: F) -> Self {
            Self(Some(f))
        }
    }

    impl<F: FnOnce() + Send + 'static> Drop for OnDrop<F> {
        fn drop(&mut self) {
            if let Some(f) = self.0.take() {
                f();
            }
        }
    }

    /// The graveyard is a process-wide `OnceLock`, so whichever test initializes
    /// it first fixes the capacity for every test in this binary. Keep every call
    /// on the same value, and above the 64 pushes `concurrent_producers...` makes.
    fn ensure_graveyard() {
        init_graveyard(256);
    }

    #[test]
    fn destructor_runs_on_another_thread_never_the_caller() {
        ensure_graveyard();

        let (tx, rx) = mpsc::channel();
        let probe = OnDrop::new(move || {
            let _ = tx.send(thread::current().id());
        });

        let caller = thread::current().id();
        drop_rt_safe(probe);

        let dropped_on = rx
            .recv_timeout(Duration::from_secs(5))
            .expect("value was never dropped");
        assert_ne!(
            dropped_on, caller,
            "destructor ran on the calling (audio) thread"
        );
    }

    #[test]
    fn concurrent_producers_all_drop_off_the_caller() {
        ensure_graveyard();

        const PRODUCERS: usize = 64;
        let (tx, rx) = mpsc::channel();

        let handles: Vec<_> = (0..PRODUCERS)
            .map(|_| {
                let tx = tx.clone();
                thread::spawn(move || {
                    drop_rt_safe(OnDrop::new(move || {
                        let _ = tx.send(thread::current().id());
                    }));
                })
            })
            .collect();
        drop(tx);

        let dropped_on: Vec<ThreadId> = rx.iter().take(PRODUCERS).collect();
        for handle in handles {
            handle.join().unwrap();
        }

        let caller = thread::current().id();
        assert_eq!(
            dropped_on.len(),
            PRODUCERS,
            "some values were never dropped"
        );
        assert!(
            dropped_on.iter().all(|&id| id != caller),
            "a destructor ran on a calling thread"
        );
    }

    #[test]
    fn value_needing_no_drop_is_a_no_op() {
        // No destructor to defer, so it must not touch the graveyard at all.
        drop_rt_safe(42u32);
    }

    #[test]
    fn full_queue_drops_on_the_caller() {
        // Rendezvous channel, no parked receiver: `try_send` always fails.
        let (tx, _rx) = bounded::<Garbage>(0);

        let (signal_tx, signal_rx) = mpsc::channel();
        let probe = Garbage::new(OnDrop::new(move || {
            let _ = signal_tx.send(thread::current().id());
        }));

        let caller = thread::current().id();
        dispose(Some(&tx), probe);

        assert_eq!(
            signal_rx.try_recv().ok(),
            Some(caller),
            "value should be dropped synchronously on the caller"
        );
    }

    #[test]
    fn uninitialized_graveyard_drops_on_the_caller() {
        let (signal_tx, signal_rx) = mpsc::channel();
        let probe = Garbage::new(OnDrop::new(move || {
            let _ = signal_tx.send(thread::current().id());
        }));

        let caller = thread::current().id();
        dispose(None, probe);

        assert_eq!(signal_rx.try_recv().ok(), Some(caller));
    }

    #[repr(align(16))]
    #[allow(dead_code)] // Written to the graveyard, never read back.
    struct MaxSlot([u8; SLOT_SIZE]);

    static MAX_SLOT_DROPS: AtomicU32 = AtomicU32::new(0);

    impl Drop for MaxSlot {
        fn drop(&mut self) {
            MAX_SLOT_DROPS.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn slot_accepts_a_value_exactly_at_the_size_and_align_limits() {
        // Compiles only because the const asserts in `Garbage::new` accept it.
        assert_eq!(size_of::<MaxSlot>(), SLOT_SIZE);
        assert_eq!(align_of::<MaxSlot>(), SLOT_ALIGN);

        ensure_graveyard();
        drop_rt_safe(MaxSlot([0; SLOT_SIZE]));

        for _ in 0..1000 {
            if MAX_SLOT_DROPS.load(Ordering::SeqCst) == 1 {
                return;
            }
            thread::sleep(Duration::from_millis(1));
        }
        panic!("a value at the slot limits was never dropped");
    }

    #[test]
    fn drop_rt_safe_does_not_allocate_on_the_caller() {
        ensure_graveyard();

        // Allocated before the checked block: only the RT call has to be clean.
        let payload = Box::new([0u8; 1024]);

        assert_no_alloc::assert_no_alloc(|| {
            drop_rt_safe(payload);
        });
    }

    /// Panics from its destructor, on the graveyard thread.
    struct PanicOnDrop;

    impl Drop for PanicOnDrop {
        fn drop(&mut self) {
            panic!("destructor panicked");
        }
    }

    /// A panicking destructor must abort the whole process. Unwinding would only
    /// kill the graveyard thread, after which every drop silently happens on the
    /// audio thread. The abort can't be observed in-process, so re-run this test
    /// binary as a child and check that it does not exit cleanly.
    #[test]
    fn panicking_destructor_aborts_the_process() {
        const CHILD_ENV: &str = "RIFT_GRAVEYARD_ABORT_CHILD";

        if std::env::var_os(CHILD_ENV).is_some() {
            init_graveyard(8);
            drop_rt_safe(PanicOnDrop);
            // No abort means a regression: give the graveyard thread a moment,
            // then exit cleanly so the parent fails instead of hanging.
            thread::sleep(Duration::from_secs(2));
            std::process::exit(0);
        }

        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "graveyard::tests::panicking_destructor_aborts_the_process",
            ])
            .env(CHILD_ENV, "1")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .expect("failed to re-run the test binary");

        assert!(
            !status.success(),
            "a panicking destructor must abort the process, but the child exited cleanly"
        );
    }
}
