//! [`LuaPacketValue`]: a packet built from Lua.

use mlua::{FromLua, IntoLua, Lua, Value};

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
        let body: mlua::LuaString = table.get("body")?;
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
