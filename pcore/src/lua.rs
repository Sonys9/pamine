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

use mlua::{FromLua, IntoLua, Lua, Value};

use crate::decoder::{NbtTag, Slot, Uuid, VarInt, VarLong};

// ---------------------------------------------------------------------------
// VarInt / VarLong
// ---------------------------------------------------------------------------

impl IntoLua for VarInt {
    fn into_lua(self, _: &Lua) -> mlua::Result<Value> {
        Ok(Value::Integer(self.number as i64))
    }
}

impl FromLua for VarInt {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        match value {
            Value::Integer(n) => Ok(VarInt::new(n as i32)),
            Value::Number(n) => Ok(VarInt::new(n as i32)),
            other => Err(mlua::Error::runtime(format!(
                "expected a number for VarInt, got {}",
                other.type_name()
            ))),
        }
    }
}

impl IntoLua for VarLong {
    fn into_lua(self, _: &Lua) -> mlua::Result<Value> {
        Ok(Value::Integer(self.number))
    }
}

impl FromLua for VarLong {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        match value {
            Value::Integer(n) => Ok(VarLong::new(n)),
            Value::Number(n) => Ok(VarLong::new(n as i64)),
            other => Err(mlua::Error::runtime(format!(
                "expected a number for VarLong, got {}",
                other.type_name()
            ))),
        }
    }
}

// ---------------------------------------------------------------------------
// Uuid
// ---------------------------------------------------------------------------

impl IntoLua for Uuid {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        // A Lua string keeps the 16 raw bytes intact; formatting as hex would
        // lose the exact representation and need parsing on the way back.
        Ok(Value::String(lua.create_string(self.bytes)?))
    }
}

impl FromLua for Uuid {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        let string = match value {
            Value::String(s) => s,
            other => {
                return Err(mlua::Error::runtime(format!(
                    "expected a 16-byte string for Uuid, got {}",
                    other.type_name()
                )));
            }
        };
        let bytes = string.as_bytes();
        if bytes.len() != 16 {
            return Err(mlua::Error::runtime(format!(
                "expected a 16-byte string for Uuid, got {} bytes",
                bytes.len()
            )));
        }
        let mut out = [0u8; 16];
        out.copy_from_slice(&bytes);
        Ok(Uuid::new(out))
    }
}

// ---------------------------------------------------------------------------
// NbtTag (read-only)
// ---------------------------------------------------------------------------

impl IntoLua for NbtTag<'_> {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        Ok(match self {
            NbtTag::End => Value::Nil,
            NbtTag::Byte(v) => Value::Integer(v as i64),
            NbtTag::Short(v) => Value::Integer(v as i64),
            NbtTag::Int(v) => Value::Integer(v as i64),
            NbtTag::Long(v) => Value::Integer(v),
            NbtTag::Float(v) => Value::Number(v as f64),
            NbtTag::Double(v) => Value::Number(v),
            NbtTag::ByteArray(bytes) => Value::String(lua.create_string(bytes)?),
            NbtTag::String(s) => Value::String(lua.create_string(s)?),
            NbtTag::List(items) => {
                let table = lua.create_table_with_capacity(items.len(), 0)?;
                for (i, item) in items.into_iter().enumerate() {
                    table.set(i + 1, item)?;
                }
                Value::Table(table)
            }
            NbtTag::Compound(entries) => {
                let table = lua.create_table_with_capacity(0, entries.len())?;
                for (key, value) in entries {
                    table.set(key, value)?;
                }
                Value::Table(table)
            }
            NbtTag::IntArray(values) => {
                let table = lua.create_table_with_capacity(values.len(), 0)?;
                for (i, value) in values.into_iter().enumerate() {
                    table.set(i + 1, value)?;
                }
                Value::Table(table)
            }
            NbtTag::LongArray(values) => {
                let table = lua.create_table_with_capacity(values.len(), 0)?;
                for (i, value) in values.into_iter().enumerate() {
                    table.set(i + 1, value)?;
                }
                Value::Table(table)
            }
        })
    }
}

// ---------------------------------------------------------------------------
// Slot (read-only)
// ---------------------------------------------------------------------------

impl IntoLua for Slot<'_> {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        let table = lua.create_table()?;
        table.set("present", self.present)?;
        table.set("item_id", self.item_id)?;
        table.set("count", self.count)?;
        table.set("nbt", self.nbt)?;
        Ok(Value::Table(table))
    }
}
