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

        let require = self.lua.create_function(move |lua, mut module: String| {
            {
                let cache = cache.lock().unwrap();
                if let Some(key) = cache.get(&module) {
                    return lua.registry_value(key);
                }
            }
            let code = Self::find_script(&mut module, &script)?;
            let result: LuaValue = lua.load(code).set_name(&module).eval()?;
            let key = lua.create_registry_value(result.clone())?;
            cache.lock()
                .unwrap()
                .insert(module, key);
            Ok(result)
        })?;

        self.lua.globals().set("require", require)?;
        Ok(())
    }

    /// Looks a module up in the script archive. The returned slice borrows from
    /// `script`, so the lifetime is tied to it rather than being unconstrained.
    fn find_script<'a>(path: &mut str, script: &'a Script) -> LuaResult<&'a str> {
        Self::to_path(path);
        let bytes = script
            .files
            .file(&mut *path, true)
            .ok_or_else(|| LuaError::runtime(format!("module `{path}` not found in the script")))?;
        std::str::from_utf8(bytes)
            .map_err(|e| LuaError::runtime(format!("module `{path}` is not valid UTF-8: {e}")))
    }

    fn to_path(module: &mut str) {
        let bytes = unsafe { module.as_bytes_mut() };
        for byte in bytes {
            if *byte == b'.' {
                *byte = b'/';
            };
        };
    }
}
