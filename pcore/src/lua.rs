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
// OwnedNbtTag / OwnedSlot
// ---------------------------------------------------------------------------

/// An NBT tag that owns all of its data.
///
/// [`NbtTag`] borrows from the packet buffer, which makes `FromLua` impossible:
/// a Lua string lives in Lua's GC heap and cannot be borrowed for the packet
/// lifetime. This mirror replaces every borrow with an owned `String` or
/// `Vec<u8>`, so a script can build a tag from scratch.
///
/// [`OwnedNbtTag::as_borrowed`] converts back to the zero-copy form for
/// encoding.
#[derive(Debug, Clone, PartialEq)]
pub enum OwnedNbtTag {
    End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<u8>),
    String(String),
    List(Vec<OwnedNbtTag>),
    Compound(Vec<(String, OwnedNbtTag)>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl OwnedNbtTag {
    /// Borrows this tag in the zero-copy form, for encoding into a packet.
    ///
    /// The returned tag points into `self`, so it cannot outlive it.
    pub fn as_borrowed(&self) -> NbtTag<'_> {
        match self {
            Self::End => NbtTag::End,
            Self::Byte(v) => NbtTag::Byte(*v),
            Self::Short(v) => NbtTag::Short(*v),
            Self::Int(v) => NbtTag::Int(*v),
            Self::Long(v) => NbtTag::Long(*v),
            Self::Float(v) => NbtTag::Float(*v),
            Self::Double(v) => NbtTag::Double(*v),
            Self::ByteArray(bytes) => NbtTag::ByteArray(bytes),
            Self::String(s) => NbtTag::String(s),
            Self::List(items) => NbtTag::List(items.iter().map(Self::as_borrowed).collect()),
            Self::Compound(entries) => NbtTag::Compound(
                entries
                    .iter()
                    .map(|(key, value)| (key.as_str(), value.as_borrowed()))
                    .collect(),
            ),
            Self::IntArray(values) => NbtTag::IntArray(values.clone()),
            Self::LongArray(values) => NbtTag::LongArray(values.clone()),
        }
    }
}

impl From<&NbtTag<'_>> for OwnedNbtTag {
    fn from(tag: &NbtTag<'_>) -> Self {
        match tag {
            NbtTag::End => Self::End,
            NbtTag::Byte(v) => Self::Byte(*v),
            NbtTag::Short(v) => Self::Short(*v),
            NbtTag::Int(v) => Self::Int(*v),
            NbtTag::Long(v) => Self::Long(*v),
            NbtTag::Float(v) => Self::Float(*v),
            NbtTag::Double(v) => Self::Double(*v),
            NbtTag::ByteArray(bytes) => Self::ByteArray(bytes.to_vec()),
            NbtTag::String(s) => Self::String((*s).to_owned()),
            NbtTag::List(items) => Self::List(items.iter().map(Self::from).collect()),
            NbtTag::Compound(entries) => Self::Compound(
                entries
                    .iter()
                    .map(|(key, value)| ((*key).to_owned(), Self::from(value)))
                    .collect(),
            ),
            NbtTag::IntArray(values) => Self::IntArray(values.clone()),
            NbtTag::LongArray(values) => Self::LongArray(values.clone()),
        }
    }
}

impl IntoLua for OwnedNbtTag {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        Ok(match self {
            Self::End => Value::Nil,
            Self::Byte(v) => Value::Integer(v as i64),
            Self::Short(v) => Value::Integer(v as i64),
            Self::Int(v) => Value::Integer(v as i64),
            Self::Long(v) => Value::Integer(v),
            Self::Float(v) => Value::Number(v as f64),
            Self::Double(v) => Value::Number(v),
            Self::ByteArray(bytes) => Value::String(lua.create_string(&bytes)?),
            Self::String(s) => Value::String(lua.create_string(&s)?),
            Self::List(items) => {
                let table = lua.create_table_with_capacity(items.len(), 0)?;
                for (i, item) in items.into_iter().enumerate() {
                    table.set(i + 1, item)?;
                }
                Value::Table(table)
            }
            Self::Compound(entries) => {
                let table = lua.create_table_with_capacity(0, entries.len())?;
                for (key, value) in entries {
                    table.set(key, value)?;
                }
                Value::Table(table)
            }
            Self::IntArray(values) => {
                let table = lua.create_table_with_capacity(values.len(), 0)?;
                for (i, value) in values.into_iter().enumerate() {
                    table.set(i + 1, value)?;
                }
                Value::Table(table)
            }
            Self::LongArray(values) => {
                let table = lua.create_table_with_capacity(values.len(), 0)?;
                for (i, value) in values.into_iter().enumerate() {
                    table.set(i + 1, value)?;
                }
                Value::Table(table)
            }
        })
    }
}

impl FromLua for OwnedNbtTag {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        Ok(match value {
            Value::Nil => Self::End,
            Value::Integer(n) => Self::Int(n as i32),
            Value::Number(n) => Self::Double(n),
            Value::String(s) => Self::String(
                s.to_str()
                    .map_err(|_| mlua::Error::runtime("NBT string is not valid UTF-8"))?
                    .to_owned(),
            ),
            Value::Table(table) => {
                // A table is a `Compound` when every key is a string, and a
                // `List` when the keys are exactly `1..n`. Anything else is
                // ambiguous and rejected rather than guessed at.
                let mut entries = Vec::new();
                let mut all_string_keys = true;
                let mut sequence_len = 0usize;

                for pair in table.pairs::<Value, Value>() {
                    let (key, value) = pair?;
                    match key {
                        Value::String(s) => {
                            let key = s
                                .to_str()
                                .map_err(|_| {
                                    mlua::Error::runtime("NBT compound key is not valid UTF-8")
                                })?
                                .to_owned();
                            entries.push((key, Self::from_lua(value, lua)?));
                        }
                        Value::Integer(i) => {
                            all_string_keys = false;
                            if i < 1 {
                                return Err(mlua::Error::runtime("NBT list index must be >= 1"));
                            }
                            sequence_len = sequence_len.max(i as usize);
                            entries.push((i.to_string(), Self::from_lua(value, lua)?));
                        }
                        other => {
                            return Err(mlua::Error::runtime(format!(
                                "NBT table key must be a string or an integer, got {}",
                                other.type_name()
                            )));
                        }
                    }
                }

                if all_string_keys {
                    Self::Compound(entries)
                } else {
                    if entries.len() != sequence_len {
                        return Err(mlua::Error::runtime(
                            "NBT list has holes: indices must be 1..n with no gaps",
                        ));
                    }
                    // `pairs` order is unspecified, so sort by the numeric key
                    // that was stringified above.
                    entries.sort_by_key(|(key, _)| key.parse::<usize>().unwrap_or(0));
                    Self::List(entries.into_iter().map(|(_, value)| value).collect())
                }
            }
            other => {
                return Err(mlua::Error::runtime(format!(
                    "cannot convert {} to an NBT tag",
                    other.type_name()
                )));
            }
        })
    }
}

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
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
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

// ---------------------------------------------------------------------------
// LuaPacketValue
// ---------------------------------------------------------------------------

/// A packet built from Lua: the opcode plus the already-encoded body.
///
/// Scripts cannot construct the Rust packet structs directly — most of them
/// borrow from the decode buffer (`&'a str`, `&'a [u8]`), and a Lua string
/// cannot be borrowed for that lifetime. Instead the `LuaPacket` derive
/// generates a constructor per variant that encodes the fields straight into
/// bytes, and this type carries the result.
///
/// The opcode is stored separately rather than prepended to `body` so the
/// caller can decide when to write it. `Injection::push_packet` writes
/// `opcode` then `body`, then frames the whole thing with the connection's
/// compression threshold.
#[derive(Debug, Clone)]
pub struct LuaPacketValue {
    /// The `#[id(N)]` of the variant this packet was built from.
    pub opcode: i32,
    /// The encoded fields, without the opcode and without framing.
    pub body: Vec<u8>,
}

impl IntoLua for LuaPacketValue {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        let table = lua.create_table()?;
        table.set("opcode", self.opcode)?;
        table.set("body", lua.create_string(&self.body)?)?;
        Ok(Value::Table(table))
    }
}

impl FromLua for LuaPacketValue {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        let table = match value {
            Value::Table(table) => table,
            other => {
                return Err(mlua::Error::runtime(format!(
                    "expected a packet table, got {}",
                    other.type_name()
                )));
            }
        };
        let opcode: i32 = table.get("opcode")?;
        let body: mlua::String = table.get("body")?;
        Ok(Self::new(opcode, body.as_bytes().to_vec()))
    }
}

impl LuaPacketValue {
    pub fn new(opcode: i32, body: Vec<u8>) -> Self {
        Self { opcode, body }
    }

    /// The full packet body: `opcode:VarInt` followed by the fields.
    ///
    /// This is what [`frame_packet`](crate::encoder::frame) expects, and what
    /// the peer reads after the length prefix.
    pub fn into_body(self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.body.len() + 5);
        crate::encoder::write_varint(&mut out, self.opcode);
        out.extend_from_slice(&self.body);
        out
    }
}

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
