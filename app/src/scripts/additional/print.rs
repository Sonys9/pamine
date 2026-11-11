use mlua::prelude::*;

use crate::scripts::Engine;

/// How deep `print` descends into nested tables. Guards against
/// self-referencing tables turning a `print` call into an infinite loop.
const MAX_PRINT_DEPTH: u8 = 32;

impl Engine {
    /// `print` only writes a log line, so there is nothing to await. Keeping it
    /// synchronous means the closure can capture the script by value without
    /// wrapping it in a future.
    ///
    /// The single `Arc` bump below is unavoidable: the closure must be `'static`,
    /// so it cannot borrow `self`.
    pub fn add_print(&self) -> LuaResult<()> {
        let script = self.script.clone();
        let print = self.lua.create_function(move |_, values: LuaMultiValue| {
            // Tracing automatically fixes colors in the terminal on Windows, so
            // no manual fix is required here.
            print!(
                "\x1b[2;90m{} \x1b[0;32m{}\x1b[0m: ",
                script.uuid, script.name
            );
            for value in values {
                Self::print_value(value, 0)?;
                print!(" ");
            }
            println!();
            Ok(())
        })?;

        self.lua.globals().set("print", print)
    }

    fn print_value(value: LuaValue, depth: u8) -> LuaResult<()> {
        match value {
            LuaValue::Nil => print!("nil"),
            LuaValue::Boolean(b) => print!("{b}"),
            LuaValue::Error(e) => print!("{e}"),
            LuaValue::Function(f) => print!("function at {:?}", f.to_pointer()),
            LuaValue::Integer(n) => print!("{n}"),
            LuaValue::LightUserData(ud) => print!("{ud:?}"),
            LuaValue::Number(n) => print!("{n}"),
            LuaValue::Other(_) => print!("unknown data type"),
            LuaValue::String(s) => print!("{}", s.display()),
            LuaValue::Table(t) => Self::print_table(t, depth)?,
            LuaValue::Thread(t) => print!("thread with state {:?}", t.state()),
            LuaValue::UserData(ud) => print!("{ud:?}"),
        }
        Ok(())
    }

    fn print_table(table: LuaTable, depth: u8) -> LuaResult<()> {
        if depth >= MAX_PRINT_DEPTH {
            print!("<table nested too deeply>");
            return Ok(());
        }

        let depth = depth + 1;
        println!();
        for (i, pair) in table.pairs::<LuaValue, LuaValue>().enumerate() {
            if i > 0 {
                println!();
            }
            let (key, value) = pair?;
            for _ in 0..depth {
                print!("- ");
            }
            Self::print_value(key, depth)?;
            print!(": ");
            Self::print_value(value, depth)?;
        }
        Ok(())
    }
}
