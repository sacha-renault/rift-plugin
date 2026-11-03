use std::ops::AddAssign;

pub fn sum_array_inplace<T: Copy + AddAssign, const N: usize>(arr1: &mut [T; N], arr2: &[T; N]) {
    for (a, b) in arr1.iter_mut().zip(arr2) {
        *a += *b;
    }
}
