/// A generic trait for parameters that expose their underlying type at compile time.
///
/// Useful when you need to interact with parameters of specific types (e.g., `f32` vs `bool`)
/// without casting, allowing for strongly-typed parameter sets in the UI layer.
pub trait TypedParam {
    /// The concrete type of the value held by this parameter.
    type Type;

    fn value(&self) -> Self::Type;

    fn set_value(&self, value: Self::Type);
}
