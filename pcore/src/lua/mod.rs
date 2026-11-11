//! Lua bindings for the protocol primitives.
//!
//! Enabled by the `lua` feature. These impls have to live in `pcore` rather
//! than in the consumer crate: `IntoLua`/`FromLua` are foreign traits and
//! `VarInt`/`Uuid`/`NbtTag` are foreign types to `app`, so the orphan rule
//! rejects the impls there.
//!
//! Design notes:
//!
//! - `VarInt`/`VarLong`/`Uuid` are plain values, so they map to Lua numbers and
//!   strings and round-trip losslessly.
//! - `NbtTag` and `Slot` borrow from the packet buffer (`&'a [u8]`, `&'a str`),
//!   so they are **read-only** from Lua's point of view. `FromLua` would have
//!   to invent a lifetime that outlives the Lua call, which is not possible
//!   without leaking. Scripts can inspect them but not construct them.
//!
//! Layout:
//!
//! - [`owned_nbt`] — [`OwnedNbtTag`], the owned mirror of `NbtTag`
//! - [`owned_slot`] — [`OwnedSlot`], the owned mirror of `Slot`
//! - [`packet`] — [`LuaPacketValue`], a packet built from Lua
//! - [`primitives`] — `VarInt`, `VarLong` and `Uuid` conversions
//! - [`borrowed`] — read-only `IntoLua` impls for `NbtTag` and `Slot`

mod borrowed;
mod owned_nbt;
mod owned_slot;
mod packet;
mod primitives;

pub use owned_nbt::OwnedNbtTag;
pub use owned_slot::OwnedSlot;
pub use packet::LuaPacketValue;
