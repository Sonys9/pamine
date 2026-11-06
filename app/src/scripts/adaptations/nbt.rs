//! The `nbt` global: explicit constructors for the types that inference loses.
//!
//! [`OwnedNbtTag`] itself lives in `pcore::lua`, because the `LuaPacket` derive
//! refers to it by absolute path and the orphan rule forbids the impls in
//! `app`. This module only builds the Lua namespace.
//!
//! Reading a tag is plain — no `{type = "Int", value = 5}` wrappers:
//!
//! | NBT | Lua |
//! |---|---|
//! | `End` | `nil` |
//! | `Byte`, `Short`, `Int`, `Long` | integer |
//! | `Float`, `Double` | number |
//! | `String` | string |
//! | `ByteArray` | string (raw bytes) |
//! | `List`, `IntArray`, `LongArray` | sequence table |
//! | `Compound` | table with string keys |
//!
//! Writing infers the type back: integer → `Int`, float → `Double`, string →
//! `String`, table → `Compound` when every key is a string, `List` when the
//! keys are `1..n`. That is lossy for `Byte`/`Short`/`Long`/`Float` and for
//! the array types, so this namespace exposes explicit constructors for the
//! cases where the exact wire type matters.

use mlua::{IntoLua, Lua, Value};
use pcore::lua::OwnedNbtTag;

/// The `nbt` global.
pub struct NbtBindings;

impl IntoLua for NbtBindings {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        let table = lua.create_table()?;

        table.set(
            "byte",
            lua.create_function(|_, v: i8| Ok(OwnedNbtTag::Byte(v)))?,
        )?;
        table.set(
            "short",
            lua.create_function(|_, v: i16| Ok(OwnedNbtTag::Short(v)))?,
        )?;
        table.set(
            "int",
            lua.create_function(|_, v: i32| Ok(OwnedNbtTag::Int(v)))?,
        )?;
        table.set(
            "long",
            lua.create_function(|_, v: i64| Ok(OwnedNbtTag::Long(v)))?,
        )?;
        table.set(
            "float",
            lua.create_function(|_, v: f32| Ok(OwnedNbtTag::Float(v)))?,
        )?;
        table.set(
            "double",
            lua.create_function(|_, v: f64| Ok(OwnedNbtTag::Double(v)))?,
        )?;
        table.set(
            "string",
            lua.create_function(|_, v: String| Ok(OwnedNbtTag::String(v)))?,
        )?;
        table.set(
            "byte_array",
            lua.create_function(|_, v: mlua::String| {
                Ok(OwnedNbtTag::ByteArray(v.as_bytes().to_vec()))
            })?,
        )?;
        table.set(
            "int_array",
            lua.create_function(|_, v: Vec<i32>| Ok(OwnedNbtTag::IntArray(v)))?,
        )?;
        table.set(
            "long_array",
            lua.create_function(|_, v: Vec<i64>| Ok(OwnedNbtTag::LongArray(v)))?,
        )?;
        table.set(
            "list",
            lua.create_function(|_, v: Vec<OwnedNbtTag>| Ok(OwnedNbtTag::List(v)))?,
        )?;
        table.set(
            "compound",
            lua.create_function(|_, v: mlua::Table| {
                let mut entries = Vec::new();
                for pair in v.pairs::<String, OwnedNbtTag>() {
                    entries.push(pair?);
                }
                Ok(OwnedNbtTag::Compound(entries))
            })?,
        )?;

        Ok(Value::Table(table))
    }
}
