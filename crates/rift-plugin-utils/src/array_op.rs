use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

pub fn scale<T: Copy + MulAssign>(arr: &mut [T], gain: T) {
    for v in arr.iter_mut() {
        *v *= gain;
    }
}

pub fn add_inplace<T: Copy + AddAssign>(dst: &mut [T], src: &[T]) {
    debug_assert_eq!(dst.len(), src.len());

    for (d, s) in dst.iter_mut().zip(src) {
        *d += *s;
    }
}

pub fn sub_inplace<T: Copy + SubAssign>(dst: &mut [T], src: &[T]) {
    debug_assert_eq!(dst.len(), src.len());

    for (d, s) in dst.iter_mut().zip(src) {
        *d -= *s;
    }
}

pub fn mul_inplace<T: Copy + MulAssign>(dst: &mut [T], src: &[T]) {
    debug_assert_eq!(dst.len(), src.len());

    for (d, s) in dst.iter_mut().zip(src) {
        *d *= *s;
    }
}

pub fn add_scaled<T: Copy + AddAssign + Mul<Output = T>>(dst: &mut [T], src: &[T], gain: T) {
    debug_assert_eq!(dst.len(), src.len());

    for (d, s) in dst.iter_mut().zip(src) {
        *d += *s * gain;
    }
}

pub fn offset<T: Copy + AddAssign>(arr: &mut [T], offset: T) {
    for v in arr.iter_mut() {
        *v += offset;
    }
}

/// Linear interpolation of `dst` towards `src`: `dst[i] += (src[i] - dst[i]) * t`.
pub fn lerp_inplace<T>(dst: &mut [T], src: &[T], t: T)
where
    T: Copy + Add<Output = T> + Sub<Output = T> + Mul<Output = T>,
{
    debug_assert_eq!(dst.len(), src.len());

    for (d, s) in dst.iter_mut().zip(src) {
        *d = *d + (*s - *d) * t;
    }
}

pub fn clamp<T: Copy + PartialOrd>(arr: &mut [T], min: T, max: T) {
    debug_assert!(min <= max);

    for v in arr.iter_mut() {
        if *v < min {
            *v = min;
        } else if *v > max {
            *v = max;
        }
    }
}
