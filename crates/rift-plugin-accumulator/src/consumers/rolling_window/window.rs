use rift_plugin_types::transport::BlockTime;
use rift_plugin_utils::DEFAULT_SAMPLERATE;

pub use super::bucket::Bucket;
use crate::consumers::*;
use crate::prelude::MonoConsumer;

/// Rolling history for waveform display, one bucket per display column.
///
/// The bucket size (`fine`, in samples) is derived from three settings:
/// sample rate, displayed window ([`Self::set_seconds`]) and column count
/// ([`Self::set_columns`]). You never choose it yourself.
///
/// Bucket boundaries are fixed to the absolute sample index, and a closed bucket never changes,
/// so columns are stable while they scroll (no shimmer). [`Self::columns`] also returns a phase
/// (0..1) for sub-column scrolling at sample resolution.
///
/// **Changing `fine` clears the history.** That happens whenever a setter changes the derived
/// value (typically a widget resize). Setters that leave `fine` unchanged keep the history.
///
/// Channel routing is handled upstream by `ConsumerDispatcher`.
pub struct RollingWindow<B: Bucket> {
    /// Closed buckets (ring).
    ring: Vec<B>,
    /// Number of closed buckets since the start (absolute index).
    total: u64,
    /// Partial bucket currently being filled.
    current: B,

    /// Samples per bucket, derived.
    fine: usize,
    samplerate: f64,
    seconds: f64,
    n_cols: usize,
}

impl<B: Bucket> RollingWindow<B> {
    pub fn peaks(seconds: f64) -> RollingWindow<PeakBucket> {
        RollingWindow::new(seconds)
    }

    pub fn rms(seconds: f64) -> RollingWindow<RmsBucket> {
        RollingWindow::new(seconds)
    }
}

impl<B: Bucket> RollingWindow<B> {
    pub fn new(seconds: f64) -> Self {
        let mut s = Self {
            ring: Vec::new(),
            total: 0,
            current: B::empty(),
            fine: 0,
            samplerate: DEFAULT_SAMPLERATE,
            seconds: seconds.max(f64::EPSILON),
            n_cols: 256,
        };
        s.reconfigure();
        s
    }

    /// Sets the displayed window length.
    pub fn set_seconds(&mut self, seconds: f64) {
        self.seconds = seconds.max(f64::EPSILON);
        self.reconfigure();
    }

    /// Sets the number of columns the window is spread over (typically widget width in pixels
    /// divided by the wanted column width). Clamped to at least 1.
    pub fn set_columns(&mut self, n_cols: usize) {
        self.n_cols = n_cols.max(1);
        self.reconfigure();
    }

    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        self.samplerate = sample_rate;
        self.reconfigure();
    }

    /// Samples per bucket (derived).
    pub fn fine(&self) -> usize {
        self.fine
    }

    /// Recomputes `fine` and the ring size. Reallocates and clears the history only if
    /// one of them changed.
    fn reconfigure(&mut self) {
        let window_samples = self.samplerate * self.seconds;
        let fine = (window_samples / self.n_cols as f64).floor().max(1.0) as usize;
        let visible = window_samples / fine as f64;
        // visible + 2 extra output columns + 1 slack
        let cap = visible.ceil() as usize + 3;

        if fine != self.fine || cap != self.ring.len() {
            self.fine = fine;
            self.ring = vec![B::empty(); cap];
            self.total = 0;
            self.current = B::empty();
        }
    }

    /// Exact (fractional) number of columns spanning the window. Slightly above `n_cols`
    /// because `fine` is rounded down.
    ///
    /// Use it to size columns so the window fills the widget exactly: `col_w = width / visible_columns()`.
    pub fn visible_columns(&self) -> f32 {
        (self.samplerate * self.seconds / self.fine as f64) as f32
    }

    /// Number of columns written by [`Self::columns`].
    pub fn out_len(&self) -> usize {
        self.ring.len() - 1
    }

    /// Buckets from newest (the partial one) to oldest. Yields at most `out_len()` items,
    /// fewer right after start or a reconfigure (nothing to draw further left).
    /// Use `.rev()` for oldest first.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &B> + DoubleEndedIterator + '_ {
        let cap = self.ring.len() as u64;
        let n = (self.out_len() as u64).min(self.total + 1) as usize;
        (0..n).map(move |c| {
            if c == 0 {
                &self.current
            } else {
                &self.ring[((self.total - c as u64) % cap) as usize]
            }
        })
    }

    pub fn phase(&self) -> f32 {
        self.current.count() as f32 / self.fine as f32
    }

    #[inline]
    fn close_current(&mut self) {
        let cap = self.ring.len() as u64;
        let closed = std::mem::replace(&mut self.current, B::empty());
        self.ring[(self.total % cap) as usize] = closed;
        self.total += 1;
    }
}

impl<B: Bucket> MonoConsumer for RollingWindow<B> {
    fn consume(&mut self, mut block: &[f32], info: BlockTime) {
        if info.sample_rate() != self.samplerate {
            self.set_sample_rate(info.sample_rate());
        }

        while !block.is_empty() {
            let take = self
                .fine
                .saturating_sub(self.current.count())
                .min(block.len());
            for &s in &block[..take] {
                self.current.add_sample(s);
            }
            block = &block[take..];
            if self.current.count() >= self.fine {
                self.close_current();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rift_plugin_types::transport::BlockTime;

    use crate::consumers::Bucket;
    use crate::consumers::PeakBucket;

    use super::*;

    type Wb = RollingWindow<PeakBucket>;

    /// Sample rate used by `feed_block`, so `consume` never triggers a reconfigure.
    fn sr() -> f64 {
        BlockTime::none().sample_rate()
    }

    fn make_buffer(seconds: f64, n_cols: usize) -> Wb {
        let mut buf = Wb::new(seconds);
        buf.set_sample_rate(sr());
        buf.set_columns(n_cols);
        buf
    }

    fn feed_block(buffer: &mut Wb, block: &[f32]) {
        buffer.consume(block, BlockTime::none());
    }

    fn get_values(b: &Wb) -> (Vec<PeakBucket>, f32) {
        let out = b.iter().cloned().rev().collect();
        let phase = b.phase();
        (out, phase)
    }

    fn maxes(cols: &[PeakBucket]) -> Vec<f32> {
        cols.iter().map(|c| c.max()).collect()
    }

    /// Deterministic, non-repeating-looking signal indexed by absolute sample position.
    fn signal(range: std::ops::Range<usize>) -> Vec<f32> {
        range
            .map(|j| ((j * 37) % 101) as f32 / 100.0 - 0.5)
            .collect()
    }

    #[test]
    fn test_initial_columns_are_empty() {
        let b = make_buffer(1.0, 64);
        let (cols, phase) = get_values(&b);
        assert_eq!(cols.len(), 1); // only the partial bucket exists yet
        assert!(cols.iter().all(|c| c.count() == 0 && c.value() == 0.0));
        assert_eq!(phase, 0.0);
    }

    #[test]
    fn test_fine_is_derived_from_window_and_columns() {
        let b = make_buffer(1.0, 100);
        assert_eq!(b.fine(), (sr() / 100.0).floor() as usize);

        let n = 100.0_f32;
        let vis = b.visible_columns();
        assert!(vis >= n && vis < n + 1.0, "visible = {vis}");
    }

    #[test]
    fn test_fine_never_zero() {
        let b = make_buffer(0.001, 1_000_000);
        assert_eq!(b.fine(), 1);
        assert!(b.out_len() >= 1);
    }

    #[test]
    fn test_produces_nonzero_peaks() {
        let mut b = make_buffer(1.0, 64);
        let fine = b.fine();
        feed_block(&mut b, &vec![1.0; fine * 8 + 10]); // 8 closed buckets + 10 partial samples
        let (cols, _) = get_values(&b);
        assert!(cols.iter().any(|c| c.max() > 0.0));
        assert_eq!(cols.last().unwrap().max(), 1.0); // newest column includes the partial bucket
    }

    #[test]
    fn test_peaks_update_after_new_data() {
        let mut b = make_buffer(1.0, 64);

        feed_block(&mut b, &vec![1.0; 2048]);
        let first = maxes(&get_values(&b).0);

        // two seconds of silence push the ones far out of the window
        feed_block(&mut b, &vec![0.0; 2 * sr() as usize]);
        let second = maxes(&get_values(&b).0);

        assert_ne!(first, second, "peaks should change after new data");
        assert!(second.iter().all(|&m| m == 0.0));
    }

    #[test]
    fn test_phase_tracks_partial_bucket() {
        let mut b = make_buffer(1.0, 64);
        let fine = b.fine();
        let half = fine / 2;

        feed_block(&mut b, &vec![0.1; fine * 5 + half]);
        let expected = half as f32 / fine as f32;
        assert!((get_values(&b).1 - expected).abs() < 1e-6);
    }

    #[test]
    fn test_phase_is_continuous_across_block_sizes() {
        // result must depend only on the samples, not on how they were chunked
        let data = signal(0..5000);

        let mut a = make_buffer(1.0, 32);
        feed_block(&mut a, &data);

        let mut b = make_buffer(1.0, 32);
        for chunk in data.chunks(37) {
            feed_block(&mut b, chunk);
        }

        let (ca, pa) = get_values(&a);
        let (cb, pb) = get_values(&b);
        assert_eq!(pa, pb);
        assert_eq!(maxes(&ca), maxes(&cb));
    }

    #[test]
    fn test_columns_are_stable_while_scrolling() {
        let mut b = make_buffer(1.0, 50);
        let fine = b.fine();

        let start = fine * 80 + 17;
        feed_block(&mut b, &signal(0..start));
        let before = get_values(&b).0;

        // advance by exactly one bucket
        feed_block(&mut b, &signal(start..start + fine));
        let after = get_values(&b).0;

        // every finished column just moved one slot to the left, content unchanged
        // (the newest one was partial, so it is excluded)
        let n = before.len();
        for i in 1..n - 1 {
            assert_eq!(
                (after[i - 1].min(), after[i - 1].max()),
                (before[i].min(), before[i].max()),
                "column {i} changed while scrolling"
            );
        }
    }

    #[test]
    fn test_columns_before_stream_start_are_empty() {
        let mut b = make_buffer(1.0, 64);
        let fine = b.fine();
        feed_block(&mut b, &vec![1.0; fine * 3]);

        let (cols, _) = get_values(&b);
        let n = cols.len();
        // oldest columns predate the stream
        assert!(cols[..n - 4].iter().all(|c| c.count() == 0));
        // newest partial is empty (exact boundary), the 3 before it are full
        assert_eq!(cols[n - 1].count(), 0);
        assert!(cols[n - 4..n - 1].iter().all(|c| c.max() == 1.0));
    }

    #[test]
    fn test_old_data_falls_out_of_ring() {
        let mut b = make_buffer(0.1, 100);
        feed_block(&mut b, &vec![1.0; 100]);
        feed_block(&mut b, &vec![0.0; (sr() * 0.5) as usize]);

        let (cols, _) = get_values(&b);
        assert!(cols.iter().all(|c| c.max() == 0.0));
    }

    #[test]
    fn test_set_columns_changing_fine_clears_history() {
        let mut b = make_buffer(1.0, 64);
        feed_block(&mut b, &vec![1.0; 5000]);

        b.set_columns(128);
        assert_eq!(b.fine(), (sr() / 128.0).floor() as usize);
        let (cols, phase) = get_values(&b);
        assert!(cols.iter().all(|c| c.count() == 0));
        assert_eq!(phase, 0.0);
    }

    #[test]
    fn test_unchanged_settings_keep_history() {
        let mut b = make_buffer(1.0, 64);
        feed_block(&mut b, &vec![1.0; 5000]);

        b.set_columns(64);
        b.set_seconds(1.0);
        b.set_sample_rate(sr());
        assert!(
            get_values(&b).0.iter().any(|c| c.max() == 1.0),
            "history was lost"
        );
    }

    #[test]
    fn test_set_seconds_rescales_window() {
        let mut b = make_buffer(1.0, 100);
        let fine_1s = b.fine();

        b.set_seconds(2.0);
        assert!(b.fine() > fine_1s);
        let vis = b.visible_columns();
        assert!(vis >= 100.0 && vis < 101.0, "visible = {vis}");
    }
}
