use pcore_gen::Decode;

// Numeric slices cannot be decoded by reinterpreting bytes: the pointer would
// need to be aligned and the protocol is big-endian, so `&[u32]` is rejected.
#[derive(Decode)]
struct WithIntSlice<'a> {
    values: &'a [u32],
}

fn main() {}
