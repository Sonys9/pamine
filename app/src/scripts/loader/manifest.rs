use uuid::Uuid;

use super::tree::ScriptStruct;

/// The `[script]` table of `Script.toml`.
#[derive(Debug, Clone)]
pub struct ScriptMeta {
    pub name: String,
    pub description: String,
    pub uuid: Uuid,
    pub engine_version: u8,
    pub main_file: String,
}

impl ScriptMeta {
    /// Reads and validates `Script.toml` from the root of the script tree.
    pub fn parse(files: &ScriptStruct) -> eyre::Result<Self> {
        let raw = files
            .file("Script.toml", false)
            .ok_or_else(|| eyre::eyre!("The script archive has no `Script.toml` in its root"))?;
        let raw = std::str::from_utf8(raw)
            .map_err(|e| eyre::eyre!("`Script.toml` is not valid UTF-8: {e}"))?;
        let document: toml::Table = toml::from_str(raw)
            .map_err(|e| eyre::eyre!("Failed to parse `Script.toml`: {e}"))?;

        let script = document
            .get("script")
            .and_then(toml::Value::as_table)
            .ok_or_else(|| eyre::eyre!("`Script.toml` is missing the `[script]` table"))?;

        let required_str = |key: &str| -> eyre::Result<String> {
            script
                .get(key)
                .and_then(toml::Value::as_str)
                .map(str::to_owned)
                .ok_or_else(|| eyre::eyre!("`Script.toml`: `[script].{key}` must be a string"))
        };

        let engine_version = script
            .get("engine_version")
            .and_then(toml::Value::as_integer)
            .ok_or_else(|| {
                eyre::eyre!("`Script.toml`: `[script].engine_version` must be an integer")
            })?;
        let engine_version = u8::try_from(engine_version).map_err(|_| {
            eyre::eyre!("`Script.toml`: `[script].engine_version` must fit into a u8")
        })?;

        let uuid = required_str("uuid")?
            .parse::<Uuid>()
            .map_err(|e| eyre::eyre!("`Script.toml`: `[script].uuid` is not a valid UUID: {e}"))?;

        Ok(Self {
            name: required_str("name")?,
            description: script
                .get("description")
                .and_then(toml::Value::as_str)
                .unwrap_or_default()
                .to_owned(),
            uuid,
            engine_version,
            main_file: required_str("main_file")?,
        })
    }
}
