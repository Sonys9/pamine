use std::{cell::RefCell, rc::Rc, sync::Arc, time::Duration};

use mlua::{StdLib, prelude::*, Function};
use tokio::sync::Mutex;

use crate::packets::Packet;

#[derive(Debug)]
pub struct EngineInner {
    handlers: [Option<(LuaRegistryKey, String)>; 256],
    lua: Lua,
}

#[derive(Debug, Clone)]
pub struct Engine(Arc<Mutex<EngineInner>>);

impl Engine {
    pub fn new() -> LuaResult<Self> {
        let lua = Lua::new_with(StdLib::ALL_SAFE | StdLib::JIT, LuaOptions::default())?;  

        Ok(Self(Arc::new(Mutex::new(
            EngineInner {
                handlers: [const { None }; 256],
                lua,
            }
        ))))
    }

    pub async fn set_globals(&self) -> LuaResult<()> {
        let inner = self.0.lock().await;
        let sleep_binding = inner.lua.create_async_function(|_, millis: u64| async move {
            tokio::time::sleep(Duration::from_millis(millis)).await;
            Ok(())
        })?;

        inner.lua.globals().set("sleep", sleep_binding)?;
        inner.lua.globals().set("engine", self.clone())?;
        Ok(())
    }

    pub async fn run(&self, code: &str) -> LuaResult<()> {
        self.0.lock().await.lua.load(code).exec_async().await
    }

    pub async fn register(&self, packet_id: u8, own_type: String, func: Function) -> LuaResult<()> {
        let mut inner = self.0.lock().await;
        let key = inner.lua.create_registry_value(func)?;
        inner.handlers[packet_id as usize] = Some((key, own_type));
        Ok(())
    }

    pub async fn on_packet(&self, packet_id: u8, packet: String) -> LuaResult<()> {
        let inner = self.0.lock().await;
        let Some((ref registry_key, ref own_type)) = inner.handlers[packet_id as usize] else {
            return Ok(());
        };
        let func: Function = inner.lua.registry_value(registry_key)?;
        func.call_async::<()>(packet).await?;
        Ok(())
    }
}

impl LuaUserData for Engine {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_async_method_mut("register", |_lua, this, (packet_id, own_type, func): (u8, String, Function)| {
            let this = this.clone(); // Doesn't really cost anything (that's an Arc<...>), but that's still an allocation
            async move {
                this.register(packet_id, own_type, func).await
            }
        });
    }
}