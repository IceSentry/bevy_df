use std::ops::Sub;

#[allow(unused)]
pub fn lerp<T: num::Float + Sub>(a: T, b: T, v: T) -> T {
    (T::one() - v) * a + b * v
}

#[allow(unused)]
pub fn inverse_lerp<T: num::Float + Sub>(a: T, b: T, v: T) -> T {
    (v - a) / (b - a)
}
