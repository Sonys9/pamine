//! Block and registry tables, generated at compile time.
//!
//! `build.rs` parses `data/blocks.json` and `data/registries.json` and writes
//! two files into `$OUT_DIR`, both included below. Nothing is read from disk at
//! runtime.
//!
//! - `blocks_gen.rs` — the 933 blocks with their physics, states and drops.
//! - `registries_gen.rs` — all 63 registries from the server's data generator
//!   report, plus the `Item` type that block drops resolve to.

#[allow(dead_code, unused_imports)]
mod generated {
    include!(concat!(env!("OUT_DIR"), "/registries_gen.rs"));
    include!(concat!(env!("OUT_DIR"), "/blocks_gen.rs"));
}

#[allow(unused_imports)]
pub use generated::*;

#[cfg(test)]
mod tests;
