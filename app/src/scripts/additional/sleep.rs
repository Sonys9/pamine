use std::time::Duration;

use mlua::prelude::*;

use crate::scripts::Engine;

impl Engine {
    pub fn add_sleep(&self) -> LuaResult<()> {
        let sleep = self
            .lua
            .create_async_function(|_, millis: u64| async move {
                tokio::time::sleep(Duration::from_millis(millis)).await;
                Ok(())
            })?;

        self.lua.globals().set("sleep", sleep)
    }
}
