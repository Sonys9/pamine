use std::{
    io::{Cursor, Read},
    path::{Component, Path},
};

use tar::Archive;
use uuid::Uuid;

use super::{manifest::ScriptMeta, tree::ScriptStruct};

/// A parsed script archive.
///
/// The archive is a `.tar` with a `Script.toml` manifest in its root and the
/// Lua sources next to it. Everything is kept in memory: scripts are small and
/// the engine needs random access to the files anyway.
#[derive(Debug, Clone)]
pub struct Script {
    pub name: String,
    pub description: String,
    pub uuid: Uuid,
    pub engine_version: u8,
    pub main_file: String,
    pub files: ScriptStruct,
}

impl Script {
    pub fn parse(script_archive: Vec<u8>) -> eyre::Result<Self> {
        let files = Self::read_archive(script_archive)?;
        let meta = ScriptMeta::parse(&files)?;

        if files.file(&meta.main_file, false).is_none() {
            eyre::bail!(
                "Script.toml points to `{}` as the main file, but the archive does not contain it",
                meta.main_file
            );
        }

        Ok(Self {
            name: meta.name,
            description: meta.description,
            uuid: meta.uuid,
            engine_version: meta.engine_version,
            main_file: meta.main_file,
            files,
        })
    }

    /// The Lua source of the script's entry point.
    pub fn main_code(&self) -> eyre::Result<&str> {
        let bytes = self.files.file(&self.main_file, false).ok_or_else(|| {
            eyre::eyre!(
                "The main file `{}` is missing from the script",
                self.main_file
            )
        })?;
        std::str::from_utf8(bytes)
            .map_err(|e| eyre::eyre!("The main file `{}` is not valid UTF-8: {e}", self.main_file))
    }

    /// Unpacks the tar into a sorted [`ScriptStruct`] tree.
    fn read_archive(script_archive: Vec<u8>) -> eyre::Result<ScriptStruct> {
        let mut archive = Archive::new(Cursor::new(script_archive));
        let mut root = ScriptStruct::Folder {
            name: String::new(),
            content: Vec::new(),
        };

        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?.into_owned();
            let components = Self::normalize_path(&path)?;
            if components.is_empty() {
                continue; // the archive root itself (`./` or `.`)
            }

            let entry_type = entry.header().entry_type();
            let (is_dir, content) = if entry_type.is_dir() {
                (true, Vec::new())
            } else if entry_type.is_file() {
                let mut buffer = Vec::new();
                entry.read_to_end(&mut buffer)?;
                (false, buffer)
            } else {
                // Symlinks, hardlinks, devices, ... have no meaning inside a script.
                continue;
            };

            root.insert(&components, content, is_dir)?;
        }

        let mut files = Self::unwrap_archive_root(root)?;
        files.sort();
        Ok(files)
    }

    /// Splits an archive path into its components, rejecting anything that
    /// could escape the script root (`..`, absolute paths, Windows prefixes).
    pub(super) fn normalize_path(path: &Path) -> eyre::Result<Vec<String>> {
        let mut components = Vec::new();
        for component in path.components() {
            match component {
                Component::Normal(part) => components.push(part.to_string_lossy().into_owned()),
                Component::CurDir => {}
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                    eyre::bail!("Script archive contains an unsafe path: {}", path.display())
                }
            }
        }
        Ok(components)
    }

    /// Archives are usually created with a single top-level directory
    /// (`test_script/...`). Drop it so the paths inside the script are relative
    /// to the script root.
    pub(super) fn unwrap_archive_root(root: ScriptStruct) -> eyre::Result<ScriptStruct> {
        let ScriptStruct::Folder { content, .. } = root else {
            eyre::bail!("The script archive root is not a directory");
        };

        // Only unwrap when the single top-level directory actually holds the
        // manifest; otherwise the manifest is already at the archive root.
        if let [ScriptStruct::Folder { .. }] = content.as_slice()
            && content[0].file("Script.toml", false).is_some()
        {
            return Ok(content.into_iter().next().expect("length checked above"));
        }

        Ok(ScriptStruct::Folder {
            name: String::new(),
            content,
        })
    }
}
