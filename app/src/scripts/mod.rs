//! The scripting subsystem.
//!
//! A script is a `.tar` archive holding a `Script.toml` manifest and the Lua
//! sources. [`Script`] parses the archive into an in-memory tree, [`Engine`]
//! runs the sources and dispatches packets to the handlers they registered.
//!
//! Layout:
//! - [`loader`] — everything about reading a script archive
//!   - [`loader::script`] — the archive itself (tar unpacking, path validation)
//!   - [`loader::tree`] — the in-memory file tree
//!   - [`loader::manifest`] — the `[script]` table of `Script.toml`
//! - [`engine`] — the Lua runtime, packet dispatch and [`HandlerOptions`]
//! - [`own_type`] — what a handler is allowed to do with a packet
//! - [`adaptations`] — Lua bindings for protocol types
//! - [`additional`] — small globals (`print`, `sleep`, `require`)

pub mod adaptations;
pub mod engine;
pub mod loader;
mod additional;
mod own_type;

pub use engine::{Engine, HandlerOptions};
pub use loader::Script;
pub use own_type::OwnType;

#[cfg(test)]
mod tests;
