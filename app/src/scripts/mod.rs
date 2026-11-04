use std::time::Duration;

use mlua::{StdLib, prelude::*};

/// Runs a Lua script to completion.
///
/// Currently unused: the call in `main` is commented out while the script host
/// is being designed. Kept so the sandbox setup is not lost.
#[allow(dead_code)]
pub async fn run(code: &str) -> LuaResult<()> {
    let lua = Lua::new_with(StdLib::ALL_SAFE | StdLib::JIT, LuaOptions::default())?;

    let sleep_binding = lua.create_async_function(|_, millis: u64| async move {
        tokio::time::sleep(Duration::from_millis(millis)).await;
        Ok(())
    })?;

    lua.globals().set("sleep", sleep_binding)?;

    lua.load(code).exec_async().await?;
    Ok(())
}
