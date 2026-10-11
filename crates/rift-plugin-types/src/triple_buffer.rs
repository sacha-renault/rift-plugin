use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicU8, Ordering};

const IDX: u8 = 0b011;
const NEW: u8 = 0b100;

/// Wait-free SPSC data structure.
pub struct TripleBuffer<T> {
    slots: [UnsafeCell<T>; 3],
    shared: AtomicU8,
    write_idx: AtomicU8,
    read_idx: AtomicU8,
}

unsafe impl<T: Send> Sync for TripleBuffer<T> {}

impl<T: Copy> TripleBuffer<T> {
    pub fn new(initial: T) -> Self {
        Self {
            slots: [
                UnsafeCell::new(initial),
                UnsafeCell::new(initial),
                UnsafeCell::new(initial),
            ],
            write_idx: AtomicU8::new(0),
            shared: AtomicU8::new(1),
            read_idx: AtomicU8::new(2),
        }
    }

    /// Publishes `value`, replacing any value not read yet.
    ///
    /// # Safety
    /// Must not be called concurrently with another `write`.
    pub unsafe fn write(&self, value: T) {
        let w = self.write_idx.load(Ordering::Relaxed);

        // SAFETY: slot `w` is owned by the writer.
        unsafe { *self.slots[w as usize].get() = value };

        let old = self.shared.swap(w | NEW, Ordering::AcqRel);
        self.write_idx.store(old & IDX, Ordering::Relaxed);
    }

    /// Returns the latest published value.
    ///
    /// # Safety
    /// Must not be called concurrently with another `read`.
    pub unsafe fn read(&self) -> T {
        let mut r = self.read_idx.load(Ordering::Relaxed);

        if self.shared.load(Ordering::Relaxed) & NEW != 0 {
            r = self.shared.swap(r, Ordering::AcqRel) & IDX;
            self.read_idx.store(r, Ordering::Relaxed);
        }

        // SAFETY: slot `r` is owned by the reader.
        unsafe { *self.slots[r as usize].get() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn reads_initial_value() {
        let tb = TripleBuffer::new(7);
        assert_eq!(unsafe { tb.read() }, 7);
    }

    #[test]
    fn reads_latest_write() {
        let tb = TripleBuffer::new(0);
        unsafe {
            tb.write(1);
            tb.write(2);
            tb.write(3);
            assert_eq!(tb.read(), 3);
        }
    }

    #[test]
    fn read_without_new_write_is_stable() {
        let tb = TripleBuffer::new(0);
        unsafe {
            tb.write(5);
            assert_eq!(tb.read(), 5);
            assert_eq!(tb.read(), 5);
        }
    }

    #[test]
    fn interleaved_writes_and_reads() {
        let tb = TripleBuffer::new(0);
        for i in 1..100 {
            unsafe {
                tb.write(i);
                assert_eq!(tb.read(), i);
            }
        }
    }

    #[test]
    fn concurrent_reads_are_never_torn() {
        const N: u64 = 200_000;
        let tb = Arc::new(TripleBuffer::new((0u64, 0u64)));

        let writer = {
            let tb = tb.clone();
            std::thread::spawn(move || {
                for i in 1..=N {
                    unsafe { tb.write((i, i.wrapping_mul(31))) };
                }
            })
        };

        let mut last = 0;
        while last < N {
            let (a, b) = unsafe { tb.read() };
            assert_eq!(b, a.wrapping_mul(31), "torn read");
            assert!(a >= last, "went back in time");
            last = a;
        }

        writer.join().unwrap();
    }
}
