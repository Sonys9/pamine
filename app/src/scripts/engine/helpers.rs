use mlua::prelude::*;

use crate::{packets::{Side, State}, scripts::{Engine, OwnType, engine::{FunctionData, SIDES}}};

impl Engine {
    pub fn calc_index(packet_id: u8, side: Side, state: State) -> usize {
        let state_id = u8::try_from(state).unwrap() as usize;
        let side_id = u8::try_from(side).unwrap() as usize;
        (side_id * 256) + (state_id * 256 * SIDES) + packet_id as usize
    }

    pub fn parse_packet_id(state: State, side: Side, packet: LuaValue) -> LuaResult<u8> {
        Ok(match packet {
            // A constructor table: read the metadata the derive stamped on it.
            LuaValue::Table(ref constructor) => {
                let opcode: i32 = constructor.get("opcode")?;
                let packet_id = u8::try_from(opcode).map_err(|_| {
                    LuaError::runtime(format!("packet opcode {opcode} does not fit in a u8"))
                })?;

                // The constructor knows its own side and state; a mismatch means
                // the script registered it under the wrong phase, which would
                // never fire. Report it rather than silently doing nothing.
                if let Some(constructor_side) = constructor.get::<Option<String>>("side")? {
                    let constructor_side =
                        Side::try_from(constructor_side.as_str()).map_err(LuaError::runtime)?;
                    if constructor_side != side {
                        return Err(LuaError::runtime(format!(
                            "packet is a {} packet, but it was registered as {}",
                            <&'static str>::from(constructor_side),
                            <&'static str>::from(side),
                        )));
                    }
                }
                if let Some(constructor_state) = constructor.get::<Option<String>>("state")? {
                    let constructor_state =
                        State::try_from(constructor_state.as_str()).map_err(LuaError::runtime)?;
                    if constructor_state != state {
                        return Err(LuaError::runtime(format!(
                            "packet belongs to the {constructor_state:?} state, but it was registered as {state:?}",
                        )));
                    }
                }

                packet_id
            }
            // Backwards compatibility: a bare opcode.
            LuaValue::Integer(opcode) => u8::try_from(opcode).map_err(|_| {
                LuaError::runtime(format!("packet opcode {opcode} does not fit in a u8"))
            })?,
            other => {
                return Err(LuaError::runtime(format!(
                    "expected a packet constructor (e.g. s2c.play.SetEntityMetadata), got {}",
                    other.type_name()
                )));
            }
        })
    }

    pub fn get_func(
        &self,
        packet_id: u8,
        state: State,
        side: Side,
        own_type: OwnType,
    ) -> LuaResult<Option<FunctionData>> {
        let handlers = self.handlers.read();
        let Some(ref handler) = handlers[Self::calc_index(packet_id, side, state)] else {
            return Ok(None);
        };
        if handler.own_type != own_type {
            return Ok(None);
        };
        Ok(Some(FunctionData {
            function: self.lua.registry_value(&handler.key)?,
            is_blocking: handler.is_blocking,
        }))
    }
}