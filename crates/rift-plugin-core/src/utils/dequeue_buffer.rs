use std::collections::VecDeque;

/// A ring buffer that has a flat cache where we copy the
/// values of the [`VecDeque`] to have contiguous view.
///
/// # Notes:
/// The size on heap will be twice bigger than a single [`VecDeque`],
/// but [`VecDeque::make_contiguous`] may have multiple copy pass, when [`Self::as_contiguous`]
/// always have a single copy pass.
pub struct DequeBuffer {
    inner: VecDeque<f32>,

    /// A cache were we can copy contigous version
    /// of [`VecDeque`] without multiple copy pass
    /// in the underlying data struct.
    flat_cache: Vec<f32>,
    cache_valid_length: Option<usize>,
    capacity: usize,
}

impl DequeBuffer {
    /// Create a new [`DequeBuffer`] with both ring buffer and cache buffer having `capacity` allocated.
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: VecDeque::with_capacity(capacity),
            flat_cache: vec![0.0; capacity],
            capacity,
            cache_valid_length: None,
        }
    }

    /// Add the slice into the ring buffer. Oldest values will be dropped so we don't
    /// overflow the capacity (actually won't overflow, it would just reallocate.)
    pub fn push_block(&mut self, block: &[f32]) {
        let expected_next_size = self.inner.len() + block.len();
        if expected_next_size > self.capacity {
            self.pop_n(expected_next_size - self.capacity);
        }
        self.inner.extend(block);
        self.cache_valid_length = None;
    }

    /// Get a contiguous view of the ring buffer
    #[inline]
    pub fn as_contiguous(&mut self) -> &[f32] {
        self.as_contiguous_latest(self.inner.len())
    }

    /// Get n latest element in a contiguous view
    pub fn as_contiguous_latest(&mut self, n: usize) -> &[f32] {
        if let Some(valid_length) = self.cache_valid_length
            && valid_length >= n
        {
            let start = valid_length - n;
            return &self.flat_cache[start..start + n];
        }

        let (front, back) = self.inner.as_slices();
        let total_available = front.len() + back.len();

        // We only want the 'n' most recent samples.
        // In a VecDeque, 'back' contains the newest samples.
        let to_copy = n.min(total_available);
        let mut remaining = to_copy;
        let mut write_ptr = to_copy;

        // the newest data
        let back_len = back.len();
        let from_back = back_len.min(remaining);
        write_ptr -= from_back;
        self.flat_cache[write_ptr..write_ptr + from_back]
            .copy_from_slice(&back[back_len - from_back..]);
        remaining -= from_back;

        // If we still need more, take from the end of the 'front' slice
        if remaining > 0 {
            let front_len = front.len();
            let from_front = front_len.min(remaining);
            write_ptr -= from_front;
            self.flat_cache[write_ptr..write_ptr + from_front]
                .copy_from_slice(&front[front_len - from_front..]);
        }

        self.cache_valid_length = Some(to_copy);
        &self.flat_cache[..to_copy]
    }

    /// Return the current len of the ring buffer
    ///
    /// Will always be <= than capacity.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns true if the buffer contains no elements
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Clears the buffer, removing all values
    pub fn clear(&mut self) {
        self.inner.clear();
        self.cache_valid_length = None;
    }

    /// Remove n oldest elements
    pub fn pop_n(&mut self, to_remove: usize) {
        self.inner.drain(0..to_remove.min(self.inner.len()));
        if let Some(valid_length) = self.cache_valid_length {
            self.cache_valid_length = valid_length.checked_sub(to_remove);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_reuse() {
        let mut buf = DequeBuffer::new(10);
        buf.push_block(&[1.0, 2.0, 3.0, 4.0, 5.0]);

        // fill the cache
        let full = buf.as_contiguous_latest(5);
        assert_eq!(full, &[1.0, 2.0, 3.0, 4.0, 5.0]);

        // reuse the cache
        assert_eq!(buf.cache_valid_length, Some(5));
        let partial = buf.as_contiguous_latest(3);
        assert_eq!(partial, &[3.0, 4.0, 5.0]);
    }

    #[test]
    fn test_wraparound() {
        let mut buf = DequeBuffer::new(5);
        buf.push_block(&[1.0, 2.0, 3.0]);
        buf.push_block(&[4.0, 5.0, 6.0]); // should eject one

        let result = buf.as_contiguous();
        assert_eq!(result, &[2.0, 3.0, 4.0, 5.0, 6.0]);
    }

    #[test]
    fn test_clear() {
        let mut buf = DequeBuffer::new(5);
        buf.push_block(&[1.0, 2.0, 3.0]);
        buf.push_block(&[4.0, 5.0, 6.0]);

        let result = buf.as_contiguous();
        assert_eq!(result, &[2.0, 3.0, 4.0, 5.0, 6.0]);
        assert!(buf.cache_valid_length.is_some());
        buf.clear();
        assert!(buf.cache_valid_length.is_none());
        assert!(buf.as_contiguous().is_empty());
    }

    #[test]
    fn test_pop_n() {
        let mut buf = DequeBuffer::new(5);
        buf.push_block(&[1.0, 2.0, 3.0]);
        buf.push_block(&[4.0, 5.0, 6.0]);

        let result = buf.as_contiguous();
        assert_eq!(result, &[2.0, 3.0, 4.0, 5.0, 6.0]);
        assert!(buf.cache_valid_length.is_some());

        buf.pop_n(3);
        assert_eq!(buf.cache_valid_length, Some(2));
        assert_eq!(buf.as_contiguous().len(), 2);

        buf.pop_n(3); // More than cap
        assert!(buf.cache_valid_length.is_none());
        assert!(buf.as_contiguous().is_empty());
    }
}
