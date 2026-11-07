use std::time::Duration;

use mlua::{prelude::*};
use tracing::info;

pub fn sleep(lua: &Lua) -> LuaResult<()> {
    let sleep_binding = lua
        .create_async_function(|_, millis: u64| async move {
            tokio::time::sleep(Duration::from_millis(millis)).await;
            Ok(())
        })?;

    lua.globals().set("sleep", sleep_binding)
}

pub fn print(lua: &Lua) -> LuaResult<()> {
    let print_binding = lua
        .create_async_function(|_, text: String| async move {
            info!("[SCRIPT NAME] {}", text);
            Ok(())
        })?;

    lua.globals().set("print", print_binding)
}