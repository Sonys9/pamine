use std::sync::Arc;

use crate::scripts::{Engine, Script};

#[derive(Debug, Default)]
pub struct ScriptStorage {
    scripts: Vec<Arc<ScriptData>>,
}

impl ScriptStorage {
    pub fn insert(&mut self, script_data: Arc<ScriptData>) -> eyre::Result<()> {
        let uuid = script_data.script.uuid;
        if self.scripts.iter().any(|script_data| script_data.script.uuid == uuid) {
            eyre::bail!("Failed to insert the script: UUID is not unique");
        };
        self.scripts.push(script_data);
        Ok(())
    }

    pub fn scripts(&self) -> impl Iterator<Item = &Arc<ScriptData>> {
        self.scripts.iter()
    }
}


#[derive(Debug)]
pub struct ScriptData {
    pub script: Arc<Script>,
    pub engine: Engine,
}