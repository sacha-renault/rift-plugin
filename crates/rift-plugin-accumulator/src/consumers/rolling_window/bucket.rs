/// A trait that allows [`super::window::WindowBuckets`] to works
/// with any kind of bucket that implement [`Bucket`].
///
/// See the implementation of [`super::peaks::PeakBucket`] as example.
pub trait Bucket: Clone + 'static {
    /// Create a new bucket with initial value of `x`. Right after this call
    /// [`Self::count`] must be 1.
    fn new(x: f32) -> Self;

    /// Create a new empty bucket. Right after this call
    /// [`Self::count`] must be 0.
    fn empty() -> Self;

    /// Add a sample in the current bucket. [`Self::count`] must be
    /// incremented by 1.
    fn add_sample(&mut self, x: f32);

    /// Get the current value of the bucket.
    fn value(&self) -> f32;

    /// Get the number of sample that were accumulated in this bucket.
    fn count(&self) -> usize;

    /// Merge is needed if UI columns aren't consumed at constant speed.
    /// For example, the UI view might advance most of the time by 1 columns,
    /// but wraps sometimes and consume 2. Creating a jittery UI.
    fn merge(&mut self, other: &Self);
}
