use mlua::{Function, Value::Table, prelude::*};

use crate::{
    packets::{Packet, Side, State, UnsureThreshold},
    scripts::{Engine, OwnType},
};

impl LuaUserData for Engine {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("register", |lua, this, (options, func): (LuaValue, Function)| {
            let register_options = RegisterOptions::from_lua(options, lua)?;
            this.register(register_options, func)
        });
    }
}

pub struct RegisterOptions {
    pub state: State,
    pub side: Side,
    pub packet: LuaValue,
    pub own_type: OwnType,
    pub is_blocking: bool,
}

impl FromLua for RegisterOptions {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        let Table(table) = value else {
            return Err(LuaError::runtime("Invalid options type, expected Table"));
        };
        let state: String = table.get("state")?;
        let side: String = table.get("side")?;
        let packet: LuaValue = table.get("packet")?;
        let own_type: String = table.get("own_type")?;
        let is_blocking: bool = table.get("is_blocking")?;
        Ok(Self {
            state: State::try_from(state.as_str()).map_err(|e| LuaError::runtime(e))?,
            side: Side::try_from(side.as_str()).map_err(|e| LuaError::runtime(e))?,
            packet,
            own_type: OwnType::try_from(own_type.as_str()).map_err(|e| LuaError::runtime(e))?,
            is_blocking,
        })
    }
}

pub struct FunctionData {
    pub function: Function,
    pub is_blocking: bool,
}

#[derive(Debug, Clone)]
pub struct HandlerOptions<'a> {
    pub packet: &'a Packet<'a>,
    pub threshold: UnsureThreshold,
    pub is_injected: bool,
}

impl<'a> HandlerOptions<'a> {
    pub fn new(packet: &'a Packet, threshold: UnsureThreshold, is_injected: bool) -> Self {
        Self {
            packet,
            threshold,
            is_injected,
        }
    }
}

impl IntoLua for HandlerOptions<'_> {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        table.set("packet", self.packet)?;
        table.set("threshold", &self.threshold)?;
        table.set("is_injected", self.is_injected)?;
        Ok(LuaValue::Table(table))
    }
}

#[derive(Debug)]
pub struct Handler {
    pub key: LuaRegistryKey,
    pub own_type: OwnType,
    pub is_blocking: bool,
}
