use std::time::Duration;

use mlua::prelude::*;
use tracing::info;

use crate::scripts::Engine;

impl Engine {
    pub fn add_sleep(&self) -> LuaResult<()> {
        let sleep_binding = self.lua.create_async_function(|_, millis: u64| async move {
            tokio::time::sleep(Duration::from_millis(millis)).await;
            Ok(())
        })?;

        self.lua.globals().set("sleep", sleep_binding)
    }

    /// `print` only writes a log line, so there is nothing to await. Keeping it
    /// synchronous lets the closure *borrow* the captured script instead of
    /// moving it into a future — which removes both clones entirely.
    ///
    /// The single `Arc` bump below is unavoidable: the closure must be `'static`,
    /// so it cannot borrow `self`.
    pub fn add_print(&self) -> LuaResult<()> {
        let script = self.script.clone();
        let print_binding = self.lua.create_function(move |_, text: String| {
            // Tracing automaticly fixes colors in terminal for Windows so
            // It doesn't requires a manual fix
            println!("\x1b[2;90m{} \x1b[0;32m{}\x1b[0m: {}", script.uuid, script.name, text);
            Ok(())
        })?;

        self.lua.globals().set("print", print_binding)
    }
}