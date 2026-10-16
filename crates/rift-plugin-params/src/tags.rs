/// A parameter's tags: an opaque `u32` bitfield.
///
/// The bits have no meaning to the framework; a plugin defines its own flag
/// type (typically with `bitflags`) and queries parameters by it through
/// [`UserParams::params_by_tags`](crate::UserParams::params_by_tags).
#[derive(Debug, Default, Clone, Copy)]
pub struct Tags(pub u32);

impl Tags {
    /// The raw bits.
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// True if any bit of `other` is also set in `self`.
    pub fn intersects(self, other: impl Into<Tags>) -> bool {
        self.0 & other.into().0 != 0
    }

    /// True if every bit of `other` is also set in `self`.
    pub fn contains(self, other: impl Into<Tags>) -> bool {
        let other = other.into().0;
        self.0 & other == other
    }
}

/// Any `u32`-backed `bitflags` type converts to [`Tags`], so a plugin's own
/// flags can be used directly wherever a `Tags` is expected (and its
/// `intersects`/`contains` take them without a `.bits()` call).
///
/// There is deliberately no `From<u32>`: the blanket impl below and
/// `From<u32>` overlap in the compiler's eyes (upstream `bitflags` could one day
/// implement `Flags for u32`), so only one can exist. Use `Tags(bits)` for the
/// raw case.
impl<F> From<F> for Tags
where
    F: bitflags::Flags,
    F::Bits: Into<u32>,
{
    fn from(f: F) -> Self {
        Tags(f.bits().into())
    }
}
