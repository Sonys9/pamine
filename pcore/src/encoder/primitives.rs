//! Fixed-width big-endian scalar writers.
//!
//! Every integer and float on the Minecraft wire is big-endian, so these all go
//! through `to_be_bytes` rather than a raw reinterpret.

use super::encoder::Scalar;

/// Appends a fixed-width big-endian scalar.
pub fn write_scalar<T: Scalar>(out: &mut Vec<u8>, value: T) {
    value.write_to(out);
}

/// Implements `Encode` for the fixed-width scalars.
///
/// All of them share the same body, so the macro keeps the list in one place
/// and makes it obvious that nothing is missing relative to the decoder.
macro_rules! impl_encode_scalar {
    ($($ty:ty),* $(,)?) => {
        $(
            impl crate::Encode for $ty {
                fn encode(&self, encoder: &mut crate::encoder::Encoder) {
                    encoder.scalar(*self);
                }
            }
        )*
    };
}

impl_encode_scalar!(i8, u8, i16, u16, i32, u32, i64, u64, f32, f64);

impl crate::Encode for bool {
    fn encode(&self, encoder: &mut crate::encoder::Encoder) {
        encoder.bool(*self);
    }
}
