use std::sync::atomic::{AtomicBool, Ordering};

use crossbeam_queue::ArrayQueue;

/// Stores tasks in a bounded, lock-free FIFO queue shared between plugin threads.
///
/// Posting to a full queue evicts the oldest task instead of rejecting the new
/// one; the eviction is reported by [`Self::try_post`] and latched by
/// [`Self::take_overflow`].
pub struct TaskQueue<T> {
    queue: ArrayQueue<T>,
    is_overflow: AtomicBool,
}

impl<T> TaskQueue<T> {
    /// Creates a queue with room for `capacity` tasks, clamped to at least one.
    pub fn new(capacity: usize) -> Self {
        Self {
            is_overflow: AtomicBool::new(false),
            queue: ArrayQueue::new(capacity.max(1)),
        }
    }

    /// Posts `task`, returning the oldest task if the queue was full.
    ///
    /// The new task is always enqueued; the returned task is the one that was
    /// evicted.
    pub fn try_post(&self, task: T) -> Option<T> {
        let dropped = self.queue.force_push(task);
        if dropped.is_some() {
            self.is_overflow.store(true, Ordering::Relaxed);
        }
        dropped
    }

    /// Removes and returns the oldest task.
    #[inline]
    pub fn pop(&self) -> Option<T> {
        self.queue.pop()
    }

    /// Returns whether a task was evicted since the last call, clearing the flag.
    #[inline]
    pub fn take_overflow(&self) -> bool {
        self.is_overflow.swap(false, Ordering::Relaxed)
    }

    /// Posts `task`, silently dropping the oldest task if the queue is full.
    ///
    /// Use [`Self::try_post`] when the caller needs the evicted task.
    #[inline]
    pub fn post(&self, task: T) {
        _ = self.try_post(task);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pops_in_fifo_order() {
        let queue = TaskQueue::new(2);

        assert_eq!(queue.try_post(1), None);
        assert_eq!(queue.try_post(2), None);
        assert_eq!(queue.pop(), Some(1));
        assert_eq!(queue.pop(), Some(2));
        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn overflow_evicts_the_oldest_task() {
        let queue = TaskQueue::new(1);

        assert_eq!(queue.try_post(1), None);
        assert_eq!(queue.try_post(2), Some(1));
        assert!(queue.take_overflow());
        assert!(!queue.take_overflow());
        assert_eq!(queue.pop(), Some(2));
    }

    #[test]
    fn zero_capacity_holds_one_task() {
        let queue = TaskQueue::new(0);

        queue.post(1);
        assert_eq!(queue.pop(), Some(1));
        assert!(!queue.take_overflow());
    }
}
