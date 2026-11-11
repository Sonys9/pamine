use std::sync::Arc;

use mlua::{Function, StdLib, prelude::*};
use parking_lot::RwLock;
use tracing::{info, warn};

use crate::{
    connections::Update,
    packets::Side,
    scripts::{
        HandlerOptions, OwnType, Script,
        adaptations::{NbtBindings, UpdateBindings},
        engine::{ENGINE_VERSION, Handler, RegisterOptions, SIDES, STATES, packet_bindings},
    },
};

// Cheap to copy because Lua is Arc + bool
#[derive(Debug, Clone)]
pub struct Engine {
    pub(crate) handlers: Arc<RwLock<[Option<Handler>; 256 * SIDES * STATES]>>, // There is 2 sides and 4 states
    pub(crate) lua: Lua,
    pub(crate) script: Arc<Script>, // Arc<..> because Script is heavy
}

impl Engine {
    pub fn new(script: Arc<Script>) -> LuaResult<Self> {
        let lua = Lua::new_with(
            StdLib::BIT | StdLib::MATH | StdLib::STRING | StdLib::TABLE,
            LuaOptions::default(),
        )?;

        Ok(Self {
            handlers: Arc::new(RwLock::new([const { None }; 256 * 2 * 4])),
            lua,
            script,
        })
    }

    pub fn set_globals(&self) -> LuaResult<()> {
        self.lua.globals().set("engine", self.clone())?;
        self.lua.globals().set("update", UpdateBindings)?;
        self.lua.globals().set("nbt", NbtBindings)?;
        self.lua
            .globals()
            .set("s2c", packet_bindings(&self.lua, Side::S2C)?)?;
        self.lua
            .globals()
            .set("c2s", packet_bindings(&self.lua, Side::C2S)?)?;

        self.add_sleep()?;
        self.add_print()?;
        self.add_require()?;

        Ok(())
    }

    pub async fn run(&self, code: &str) -> LuaResult<()> {
        info!("Loading script \"{}\"", self.script.name);
        info!("{}", self.script.description);
        match self.script.engine_version {
            ver if ver < ENGINE_VERSION => {
                return Err(LuaError::external(format!(
                    "The engine is outdated: the script is for engine v{}, current version is {}",
                    self.script.engine_version, ENGINE_VERSION
                )));
            }
            ver if ver > ENGINE_VERSION => warn!(
                "The script is outdated: the script is for engine v{}, current version is {}",
                self.script.engine_version, ENGINE_VERSION
            ),
            _ => {}
        };
        self.lua.load(code).exec_async().await
    }

    pub async fn on_packet(
        &self,
        func: Function,
        own_type: OwnType,
        packet_id: u8,
        options: HandlerOptions<'_>,
    ) -> LuaResult<Option<Option<Update>>> {
        let result = func.call_async::<Option<Update>>(options).await?;
        Ok(Some(match (result, own_type) {
            // Returning nothing means "leave the packet alone" and is always allowed.
            (None, _) => None,

            // Read-only must not touch the packet in any way.
            (Some(_), OwnType::ReadOnly) => {
                return Err(LuaError::runtime(format!(
                    "handler for packet {packet_id} is registered as \"read\" but returned an Update"
                )));
            }

            // Read-write may replace, but not inject into the other direction.
            (Some(update), OwnType::ReadWrite) if update.inject.is_some() => {
                return Err(LuaError::runtime(format!(
                    "handler for packet {packet_id} is registered as \"read-write\" but returned an inject"
                )));
            }
            (Some(update), OwnType::ReadWrite) => Some(update),

            // Read-inject may inject, but not replace.
            (Some(update), OwnType::ReadInject) if update.change_with.is_some() => {
                return Err(LuaError::runtime(format!(
                    "handler for packet {packet_id} is registered as \"read-inject\" but returned a change_with"
                )));
            }
            (Some(update), OwnType::ReadInject) => Some(update),

            // Full rights: anything goes.
            (Some(update), OwnType::ReadWriteInject) => Some(update),
        }))
    }

    /// Registers a handler for one packet.
    ///
    /// `packet` is a constructor from the `s2c` / `c2s` namespaces, e.g.
    /// `s2c.play.SetEntityMetadata`. The opcode, side and state are read off
    /// that constructor, so a script never has to hard-code a numeric id:
    ///
    /// ```lua
    /// engine:register("Play", "S2C", s2c.play.SetEntityMetadata, "read", handler)
    /// ```
    ///
    /// The `state` and `side` arguments are still accepted (and checked against
    /// the constructor) so a typo is reported instead of silently registering
    /// the handler under the wrong phase. A plain integer is also accepted for
    /// backwards compatibility.
    pub fn register(&self, register_options: RegisterOptions) -> LuaResult<()> {
        let packet_id = Self::parse_packet_id(
            register_options.state,
            register_options.side,
            register_options.packet,
        )?;
        let key = self.lua.create_registry_value(register_options.function)?;

        self.handlers.write()
            [Self::calc_index(packet_id, register_options.side, register_options.state)] =
            Some(Handler {
                key,
                own_type: register_options.own_type,
                is_blocking: register_options.is_blocking,
            });
        Ok(())
    }
}
