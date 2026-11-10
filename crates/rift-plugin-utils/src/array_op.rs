use std::ops::{AddAssign, MulAssign};

pub fn sum_array_inplace<T: Copy + AddAssign, const N: usize>(arr1: &mut [T; N], arr2: &[T; N]) {
    for (a, b) in arr1.iter_mut().zip(arr2) {
        *a += *b;
    }
}

pub fn inplace_gain<T: Copy + MulAssign>(arr: &mut [T], gain: T) {
    for v in arr.iter_mut() {
        *v *= gain;
    }
}
