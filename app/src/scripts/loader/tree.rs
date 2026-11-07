use std::{
    cmp::Ordering,
    path::{Component, Path},
};

/// A node of the in-memory script tree.
///
/// Paths are implicit: a node only knows its own name, and its position is
/// determined by the folders it is nested in.
#[derive(Debug, Clone)]
pub enum ScriptStruct {
    Folder { name: String, content: Vec<ScriptStruct> },
    File { name: String, content: Vec<u8> },
}

impl ScriptStruct {
    pub fn name(&self) -> &str {
        match self {
            ScriptStruct::Folder { name, .. } | ScriptStruct::File { name, .. } => name,
        }
    }

    pub fn as_folder(&self) -> Option<&[ScriptStruct]> {
        match self {
            ScriptStruct::Folder { content, .. } => Some(content),
            ScriptStruct::File { .. } => None,
        }
    }

    pub fn as_file(&self) -> Option<&[u8]> {
        match self {
            ScriptStruct::File { content, .. } => Some(content),
            ScriptStruct::Folder { .. } => None,
        }
    }

    /// Looks a node up by a `/`-separated path relative to this node.
    pub fn get(&self, path: impl AsRef<Path>) -> Option<&ScriptStruct> {
        let mut current = self;
        for component in path.as_ref().components() {
            let Component::Normal(part) = component else { continue };
            let part = part.to_string_lossy();
            let children = current.as_folder()?;
            current = children.iter().find(|child| child.name() == part.as_ref())?;
        }
        Some(current)
    }

    /// Reads a file by a `/`-separated path relative to this node.
    pub fn file(&self, path: impl AsRef<Path>) -> Option<&[u8]> {
        self.get(path)?.as_file()
    }

    /// Inserts `content` at `components`, creating intermediate folders as
    /// needed. Directory entries are only used to create the folder itself;
    /// their (empty) content is ignored.
    pub(super) fn insert(
        &mut self,
        components: &[String],
        content: Vec<u8>,
        is_dir: bool,
    ) -> eyre::Result<()> {
        let Some((name, rest)) = components.split_first() else {
            return Ok(());
        };
        let ScriptStruct::Folder { content: children, .. } = self else {
            eyre::bail!("Cannot insert `{name}` into a file");
        };

        if rest.is_empty() {
            if is_dir {
                if Self::has_file(children, name) {
                    eyre::bail!("`{name}` is both a file and a directory in the script archive");
                }
                if !Self::has_folder(children, name) {
                    children.push(ScriptStruct::Folder { name: name.clone(), content: Vec::new() });
                }
            } else {
                if Self::has_folder(children, name) {
                    eyre::bail!("`{name}` is both a file and a directory in the script archive");
                }
                // A later entry wins, which matches how tar extraction behaves.
                children.retain(|child| !matches!(child, ScriptStruct::File { name: n, .. } if n == name));
                children.push(ScriptStruct::File { name: name.clone(), content });
            }
            return Ok(());
        }

        let index = match children
            .iter()
            .position(|child| matches!(child, ScriptStruct::Folder { name: n, .. } if n == name))
        {
            Some(index) => index,
            None => {
                if Self::has_file(children, name) {
                    eyre::bail!("`{name}` is both a file and a directory in the script archive");
                }
                children.push(ScriptStruct::Folder { name: name.clone(), content: Vec::new() });
                children.len() - 1
            }
        };
        children[index].insert(rest, content, is_dir)
    }

    /// Sorts the tree so the result is deterministic regardless of the order
    /// the entries happened to be stored in the archive.
    pub(super) fn sort(&mut self) {
        let ScriptStruct::Folder { content, .. } = self else { return };
        content.sort_by(|a, b| match (a, b) {
            (ScriptStruct::Folder { name: a, .. }, ScriptStruct::Folder { name: b, .. }) => a.cmp(b),
            (ScriptStruct::Folder { .. }, ScriptStruct::File { .. }) => Ordering::Less,
            (ScriptStruct::File { .. }, ScriptStruct::Folder { .. }) => Ordering::Greater,
            (ScriptStruct::File { name: a, .. }, ScriptStruct::File { name: b, .. }) => a.cmp(b),
        });
        for child in content {
            child.sort();
        }
    }

    fn has_file(children: &[ScriptStruct], name: &str) -> bool {
        children
            .iter()
            .any(|child| matches!(child, ScriptStruct::File { name: n, .. } if n == name))
    }

    fn has_folder(children: &[ScriptStruct], name: &str) -> bool {
        children
            .iter()
            .any(|child| matches!(child, ScriptStruct::Folder { name: n, .. } if n == name))
    }
}
