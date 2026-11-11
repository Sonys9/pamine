//! Lua conversions for the plain-value protocol primitives:
//! `VarInt`, `VarLong` and `Uuid`.

use mlua::{FromLua, IntoLua, Lua, Value};

use crate::decoder::{Uuid, VarInt, VarLong};

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
