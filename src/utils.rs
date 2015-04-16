
use std::cmp::Ordering::{self, Less, Equal, Greater};
use std::f32::consts::PI;
use num::{Float, NumCast, PrimInt, ToPrimitive};

/// Clamp a value between a given min and max.
pub fn clamp<T: PartialOrd>(n: T, min: T, max: T) -> T {
    if n < min { min } else if n > max { max } else { n }
}

/// Clamp a f32 between 0f32 and 1f32.
pub fn clampf32(f: f32) -> f32 {
    if f < 0f32 { 0f32 } else if f > 1f32 { 1f32 } else { f }
}

/// Compare two f64s and return an Ordering.
pub fn compare_f64s(a: f64, b: f64) -> Ordering {
    if a > b { Greater }
    else if a < b { Less }
    else { Equal }
}

/// Convert turns to radians.
pub fn turns<F: Float + NumCast>(t: F) -> F {
    let f: F = NumCast::from(2.0 * PI).unwrap();
    f * t
}

/// Convert degrees to radians.
pub fn degrees<F: Float + NumCast>(d: F) -> F {
    d * NumCast::from(PI / 180.0).unwrap()
}

/// The modulo function.
#[inline]
pub fn modulo<I: PrimInt>(a: I, b: I) -> I {
    match a % b {
        r if (r > I::zero() && b < I::zero())
          || (r < I::zero() && b > I::zero()) => (r + b),
        r                                         => r,
    }
}

/// Modulo float.
pub fn fmod(f: f32, n: i32) -> f32 {
    let i = f.floor() as i32;
    modulo(i, n) as f32 + f - i as f32
}

/// Return the max between to floats.
pub fn max(a: f32, b: f32) -> f32 {
    if a >= b { a } else { b }
}

/// Return the min between to floats.
pub fn min(a: f32, b: f32) -> f32 {
    if a <= b { a } else { b }
}

/// Get value percentage between max and min.
pub fn percentage<T: Float + NumCast>(value: T, min: T, max: T) -> f32 {
    let v: f32 = NumCast::from(value).unwrap();
    let mn: f32 = NumCast::from(min).unwrap();
    let mx: f32 = NumCast::from(max).unwrap();
    (v - mn) / (mx - mn)
}

/// Adjust the value to the given percentage.
pub fn value_from_perc<T: Float + NumCast + ToPrimitive>(perc: f32, min: T, max: T) -> T {
    let f: f32 = (max - min).to_f32().unwrap() * perc;
    min + NumCast::from(f).unwrap()
}

/// Map a value from a given range to a new given range.
pub fn map_range<X: Float + NumCast, Y: Float + NumCast>
(val: X, in_min: X, in_max: X, out_min: Y, out_max: Y) -> Y {
    let val_f: f64 = NumCast::from(val).unwrap();
    let in_min_f: f64 = NumCast::from(in_min).unwrap();
    let in_max_f: f64 = NumCast::from(in_max).unwrap();
    let out_min_f: f64 = NumCast::from(out_min).unwrap();
    let out_max_f: f64 = NumCast::from(out_max).unwrap();
    NumCast::from(
        (val_f - in_min_f) / (in_max_f - in_min_f) * (out_max_f - out_min_f) + out_min_f
    ).unwrap()
}

/// Get a suitable string from the value, its max and the pixel range.
pub fn val_to_string<T: ToString + NumCast>
(val: T, max: T, val_rng: T, pixel_range: usize) -> String {
    let mut s = val.to_string();
    let decimal = s.chars().position(|ch| ch == '.');
    match decimal {
        None => s,
        Some(idx) => {
            // Find the minimum string length by determing
            // what power of ten both the max and range are.
            let val_rng_f: f64 = NumCast::from(val_rng).unwrap();
            let max_f: f64 = NumCast::from(max).unwrap();
            let mut n: f64 = 0.0;
            let mut pow_ten = 0.0;
            while pow_ten < val_rng_f || pow_ten < max_f {
                pow_ten = (10.0).powf(n);
                n += 1.0
            }
            let min_string_len = n as usize + 1;

            // Find out how many pixels there are to actually use
            // and judge a reasonable precision from this.
            let mut n = 1;
            while 10us.pow(n) < pixel_range { n += 1 }
            let precision = n as usize;

            // Truncate the length to the pixel precision as
            // long as this doesn't cause it to be smaller
            // than the necessary decimal place.
            let mut truncate_len = min_string_len + (precision - 1);
            if idx + precision < truncate_len { truncate_len = idx + precision }
            if s.len() > truncate_len { s.truncate(truncate_len) }
            s
        }
    }
}

