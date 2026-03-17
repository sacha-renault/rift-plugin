pub trait Bucket: Clone + 'static {
    fn new(x: f32) -> Self;
    fn empty() -> Self;
    fn add_sample(&mut self, x: f32);
    fn value(&self) -> f32;
    fn count(&self) -> usize;
}
