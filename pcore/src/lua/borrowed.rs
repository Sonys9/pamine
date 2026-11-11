//! Read-only Lua conversions for the borrowing protocol types.
//!
//! `NbtTag` and `Slot` point into the packet buffer, so they can be handed to
//! Lua but never built from it. See the module docs in [`super`].

use mlua::{IntoLua, Lua, Value};

use crate::decoder::{NbtTag, Slot};

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
