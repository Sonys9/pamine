use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use mlua::prelude::*;

use crate::scripts::{Engine, Script};

impl Engine {
    /// `require` is synchronous by nature: loading a module is a plain
    /// `load` + `eval`, there is nothing to await. Keeping it synchronous is
    /// what lets the closure borrow the captured `Arc<Script>` directly instead
    /// of cloning it on every call — an async closure would have to move an
    /// owned copy into each future.
    pub fn add_require(&self) -> LuaResult<()> {
        let script = self.script.clone();
        let cache: Arc<Mutex<HashMap<String, LuaRegistryKey>>> =
            Arc::new(Mutex::new(HashMap::new()));

        let require = self.lua.create_function(move |lua, path: String| {
            {
                let cache = cache.lock().unwrap();
                if let Some(key) = cache.get(&path) {
                    return lua.registry_value(key);
                }
            }
            let code = Self::find_script(&path, &script)?;
            let result: LuaValue = lua.load(code).set_name(&path).eval()?;
            let key = lua.create_registry_value(result.clone())?;
            cache.lock()
                .unwrap()
                .insert(path, key);
            Ok(result)
        })?;

        self.lua.globals().set("require", require)?;
        Ok(())
    }

    /// Looks a module up in the script archive. The returned slice borrows from
    /// `script`, so the lifetime is tied to it rather than being unconstrained.
    fn find_script<'a>(path: &str, script: &'a Script) -> LuaResult<&'a str> {
        let bytes = script
            .files
            .file(path)
            .ok_or_else(|| LuaError::runtime(format!("module `{path}` not found in the script")))?;
        std::str::from_utf8(bytes)
            .map_err(|e| LuaError::runtime(format!("module `{path}` is not valid UTF-8: {e}")))
    }
}
