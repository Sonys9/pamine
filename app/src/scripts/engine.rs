use std::{sync::Arc, time::Duration};

use mlua::{Function, StdLib, prelude::*};
use tokio::sync::Mutex;

use crate::{
    connections::Update,
    packets::UnsureThreshold,
    scripts::{
        OwnType,
        adaptations::{NbtBindings, UpdateBindings},
    },
};

// Cheap to copy because Lua is Arc + bool
#[derive(Debug, Clone)]
pub struct Engine {
    handlers: Arc<Mutex<[Option<(LuaRegistryKey, OwnType)>; 256]>>,
    lua: Lua,
}

impl Engine {
    pub fn new() -> LuaResult<Self> {
        let lua = Lua::new_with(StdLib::ALL_SAFE | StdLib::JIT, LuaOptions::default())?;

        Ok(Self {
            handlers: Arc::new(Mutex::new([const { None }; 256])),
            lua,
        })
    }

    pub fn set_globals(&self) -> LuaResult<()> {
        let sleep_binding = self
            .lua
            .create_async_function(|_, millis: u64| async move {
                tokio::time::sleep(Duration::from_millis(millis)).await;
                Ok(())
            })?;

        self.lua.globals().set("sleep", sleep_binding)?;
        self.lua.globals().set("engine", self.clone())?;
        self.lua.globals().set("update", UpdateBindings)?;
        self.lua.globals().set("nbt", NbtBindings)?;

        Ok(())
    }

    pub async fn run(&self, code: &str) -> LuaResult<()> {
        self.lua.load(code).exec_async().await
    }

    async fn register(&self, packet_id: u8, own_type: String, func: Function) -> LuaResult<()> {
        let key = self.lua.create_registry_value(func)?;
        let own_type = OwnType::try_from(own_type.as_str()).map_err(|e| LuaError::external(e))?;
        self.handlers.lock().await[packet_id as usize] = Some((key, own_type));
        Ok(())
    }

    pub async fn on_packet(
        &self,
        packet_id: u8,
        packet: String,
        threshold: &UnsureThreshold,
        own_type: OwnType,
    ) -> LuaResult<Option<Option<Update>>> {
        let handlers = self.handlers.lock().await;
        let Some((ref registry_key, handler_own_type)) = handlers[packet_id as usize] else {
            return Ok(None);
        };
        if handler_own_type != own_type {
            return Ok(None);
        };
        let func: Function = self.lua.registry_value(registry_key)?;
        let result = func
            .call_async::<Option<Update>>((packet, threshold))
            .await?;
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
}

impl LuaUserData for Engine {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_async_method_mut(
            "register",
            |_lua, this, (packet_id, own_type, func): (u8, String, Function)| {
                let this = this.clone(); // Doesn't really cost anything (that's an Arc<...>), but that's still an allocation
                async move { this.register(packet_id, own_type, func).await }
            },
        );
    }
}
