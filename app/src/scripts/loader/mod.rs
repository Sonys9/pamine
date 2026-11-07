//! Loading a script archive into memory.
//!
//! - [`script`] — the archive itself: tar unpacking, path validation, manifest wiring
//! - [`tree`] — the in-memory file tree
//! - [`manifest`] — the `[script]` table of `Script.toml`

pub mod manifest;
pub mod script;
pub mod tree;

pub use script::*;

#[cfg(test)]
mod tests;