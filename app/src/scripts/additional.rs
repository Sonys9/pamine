use std::{fmt::Arguments, time::Duration};

use mlua::prelude::*;
use tracing::info;

use crate::scripts::{Engine, Script};

#[macro_use]
macro_rules! luaprint {
    ($($arg:tt)*) => {
        Engine::print(format_args!($($arg)*))
    };
}

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
        let lua = self.lua.clone();
        let script = self.script.clone();
        let print_binding = self.lua.create_function(move |_, values: LuaMultiValue| {
            // Tracing does automaticly fixes colors in terminal for Windows so
            // It doesn't requires a manual fix
            print!("\x1b[2;90m{} \x1b[0;32m{}\x1b[0m: ", script.uuid, script.name);
            for value in values {
                Self::print_value(&lua, value, 0, false).ok();
                luaprint!(" ");
            };
            luaprint!("\n");
            Ok(())
        })?;

        self.lua.globals().set("print", print_binding)
    }

    fn print_value(lua: &Lua, value: LuaValue, depth: u16, print_depth: bool) -> eyre::Result<()> {
        if print_depth {
            for _ in 0..depth {
                luaprint!("- ")
            };
        };
        match value {
            LuaNil => luaprint!("nil"),
            LuaValue::Boolean(bool) => luaprint!("{}", bool),
            LuaValue::Error(e) => luaprint!("{}", e),
            LuaValue::Function(func) => luaprint!("function at {:?}", func.to_pointer()),
            LuaValue::Integer(num) => luaprint!("{}", num),
            LuaValue::LightUserData(user_data) => luaprint!("{:?}", user_data),
            LuaValue::Number(num) => luaprint!("{}", num),
            LuaValue::Other(_other) => luaprint!("unknown data type"),
            LuaValue::String(string) => luaprint!("{}", string.display()),
            LuaValue::Table(table) => Self::print_table(&lua, table, depth)?,
            LuaValue::Thread(thread) => luaprint!("thread with state {:?}", thread.state()),
            LuaValue::UserData(user_data) => luaprint!("{:?}", user_data),
        }
        Ok(())
    }
    
    fn print_table(lua: &Lua, table: LuaTable, mut depth: u16) -> eyre::Result<()> {
        depth += 1;
        luaprint!("\n");
        for (i, element) in table.pairs::<LuaValue, LuaValue>().enumerate() {
            if i > 0 {
                luaprint!("\n");
            };
            let (key, value) = element?;
            Self::print_value(lua, key, depth, true)?;
            luaprint!(": ");
            Self::print_value(lua, value, depth, false)?;          
        };
        Ok(())
    }

    fn print(data: Arguments) {
        print!("{}", format_args!("{}", data));
        // custom logic here in the future...
    }
}