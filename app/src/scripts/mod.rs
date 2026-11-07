pub mod adaptations;
pub mod engine;
mod additional;

use std::{borrow::Cow, collections::HashMap, convert::Infallible, io::{Cursor, Read}, path::PathBuf};

pub use engine::*;
use tar::{Archive, Entry};
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Script {
    pub name: String,
    pub description: String,
    pub uuid: Uuid,
    pub engine_version: u8,
    pub main_file: String,
    pub files: Vec<ScriptStruct>,
}

impl Script {
    pub fn parse(script_archive: Vec<u8>) -> eyre::Result<Self> {
        let mut archive = Archive::new(Cursor::new(script_archive));
        let mut files = Vec::new();

        for entry in archive.entries()? {
            let entry = entry?;
            let path = entry.path()?.to_path_buf();
            if path.as_os_str() == "." { continue };

            let file_or_dir = Self::file_or_dir(path, entry)?;
            match file_or_dir {
                ScriptStruct::File { path, name, content } => {
                    info!("[{:?}/{}] {}", path, name, String::from_utf8_lossy(&content))
                },
                _ => {}
            };
        };

        Ok(Self {
            name: String::from(""),
            description: String::from(""),
            uuid: Uuid::new_v4(),
            engine_version: 0,
            main_file: String::from(""),
            files,
        })
    }

    fn file_or_dir(path: PathBuf, mut entry: Entry<'_, Cursor<Vec<u8>>>) -> eyre::Result<ScriptStruct> {
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        Ok(match entry.header().entry_type().is_dir() {
            true => ScriptStruct::Folder { path, name, content: Vec::new() },
            false => {
                let mut buffer = Vec::new();
                entry.read_to_end(&mut buffer)?;
                ScriptStruct::File { path, name, content: buffer }
            }
        })
    }
}

#[derive(Debug, Clone)]
pub enum ScriptStruct {
    Folder { path: PathBuf, name: String, content: Vec<ScriptStruct> },
    File  { path: PathBuf, name: String, content: Vec<u8> }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OwnType {
    ReadOnly,
    ReadWrite,
    ReadInject,
    ReadWriteInject,
}

impl TryFrom<OwnType> for &'static str {
    type Error = Infallible;
    fn try_from(value: OwnType) -> Result<Self, Self::Error> {
        Ok(match value {
            OwnType::ReadOnly => "read",
            OwnType::ReadWrite => "read-write",
            OwnType::ReadInject => "read-inject",
            OwnType::ReadWriteInject => "read-write-inject",
        })
    }
}

impl TryFrom<&str> for OwnType {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "read" => OwnType::ReadOnly,
            "read-write" => OwnType::ReadWrite,
            "read-inject" => OwnType::ReadInject,
            "read-write-inject" => OwnType::ReadWriteInject,
            _ => return Err("Invalid OwnType"),
        })
    }
}
