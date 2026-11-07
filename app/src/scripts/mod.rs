//! The scripting subsystem.
//!
//! A script is a `.tar` archive holding a `Script.toml` manifest and the Lua
//! sources. [`Script`] parses the archive into an in-memory tree, [`Engine`]
//! runs the sources and dispatches packets to the handlers they registered.
//!
//! Layout:
//! - [`script`] — the archive itself (manifest + file tree assembly)
//! - [`tree`] — the in-memory file tree
//! - [`manifest`] — the `[script]` table of `Script.toml`
//! - [`own_type`] — what a handler is allowed to do with a packet
//! - [`engine`] — the Lua runtime and packet dispatch
//! - [`handler_options`] — what a handler receives
//! - [`adaptations`] — Lua bindings for protocol types
//! - [`additional`] — small globals (`print`, `sleep`)

pub mod adaptations;
pub mod engine;
pub mod loader;
mod additional;
mod own_type;

pub use engine::Engine;
pub use own_type::OwnType;
pub use loader::Script;

#[cfg(test)]
mod tests;
