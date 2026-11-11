//! [`OwnedSlot`]: an inventory slot that owns its NBT.

use mlua::{FromLua, IntoLua, Lua, Value};

use crate::decoder::{Slot, VarInt};

use super::OwnedNbtTag;

/// An inventory slot that owns its NBT.
///
/// The mirror of [`Slot`], for the same reason as [`OwnedNbtTag`].
#[derive(Debug, Clone, PartialEq)]
pub struct OwnedSlot {
    pub present: bool,
    pub item_id: Option<VarInt>,
    pub count: Option<i8>,
    pub nbt: Option<OwnedNbtTag>,
}

impl OwnedSlot {
    pub fn empty() -> Self {
        Self {
            present: false,
            item_id: None,
            count: None,
            nbt: None,
        }
    }

    /// Borrows this slot in the zero-copy form, for encoding.
    pub fn as_borrowed(&self) -> Slot<'_> {
        Slot {
            present: self.present,
            item_id: self.item_id,
            count: self.count,
            nbt: self.nbt.as_ref().map(OwnedNbtTag::as_borrowed),
        }
    }
}

impl From<&Slot<'_>> for OwnedSlot {
    fn from(slot: &Slot<'_>) -> Self {
        Self {
            present: slot.present,
            item_id: slot.item_id,
            count: slot.count,
            nbt: slot.nbt.as_ref().map(OwnedNbtTag::from),
        }
    }
}

impl IntoLua for OwnedSlot {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        let table = lua.create_table()?;
        table.set("present", self.present)?;
        table.set("item_id", self.item_id)?;
        table.set("count", self.count)?;
        table.set("nbt", self.nbt)?;
        Ok(Value::Table(table))
    }
}

impl FromLua for OwnedSlot {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        let table = match value {
            Value::Table(table) => table,
            other => {
                return Err(mlua::Error::runtime(format!(
                    "expected a table for Slot, got {}",
                    other.type_name()
                )));
            }
        };
        Ok(Self {
            present: table.get::<Option<bool>>("present")?.unwrap_or(false),
            item_id: table.get::<Option<VarInt>>("item_id")?,
            count: table.get::<Option<i8>>("count")?,
            nbt: table.get::<Option<OwnedNbtTag>>("nbt")?,
        })
    }
}
