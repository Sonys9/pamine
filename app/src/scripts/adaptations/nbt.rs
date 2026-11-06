//! An owned NBT dialect for scripts.
//!
//! [`NbtTag`] is zero-copy: `String(&'a str)`, `ByteArray(&'a [u8])` and
//! `Compound(Vec<(&'a str, NbtTag<'a>)>)` all borrow from the packet buffer.
//! That is the right trade-off for decoding — no allocation per tag — but it
//! makes `FromLua` impossible: a Lua string lives in Lua's GC heap and cannot
//! be borrowed for the packet's lifetime.
//!
//! [`OwnedNbtTag`] is the scripting-side mirror. Every borrow becomes an owned
//! `String` or `Vec<u8>`, so a script can build a tag from scratch and hand it
//! back. [`OwnedNbtTag::as_borrowed`] converts it to the zero-copy form when
//! the value has to be encoded into a packet.
//!
//! ## Lua representation
//!
//! Reading is plain and ergonomic — no `{type = "Int", value = 5}` wrappers:
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
//! the array types, so the `nbt` global exposes explicit constructors for the
//! cases where the exact wire type matters.

use mlua::{FromLua, IntoLua, Lua, Value};
use pcore::decoder::NbtTag;

/// An NBT tag that owns all of its data.
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
    fn from_lua(value: Value, _lua: &Lua) -> mlua::Result<Self> {
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
                            entries.push((key, Self::from_lua(value, _lua)?));
                        }
                        Value::Integer(i) => {
                            all_string_keys = false;
                            if i < 1 {
                                return Err(mlua::Error::runtime(
                                    "NBT list index must be >= 1",
                                ));
                            }
                            sequence_len = sequence_len.max(i as usize);
                            entries.push((i.to_string(), Self::from_lua(value, _lua)?));
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

/// The `nbt` global: explicit constructors for the types that inference loses.
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
            lua.create_function(|_, v: mlua::String| Ok(OwnedNbtTag::ByteArray(v.as_bytes().to_vec())))?,
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
