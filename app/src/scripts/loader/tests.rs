use std::path::Path;

use super::{script::Script, tree::ScriptStruct};

fn folder(name: &str, content: Vec<ScriptStruct>) -> ScriptStruct {
    ScriptStruct::Folder { name: name.to_owned(), content }
}

fn file(name: &str, content: &str) -> ScriptStruct {
    ScriptStruct::File { name: name.to_owned(), content: content.as_bytes().to_vec() }
}

#[test]
fn insert_is_order_independent() {
    // A file listed before its parent directory must still end up nested.
    let mut root = folder("", Vec::new());
    root.insert(&["a".into(), "b".into(), "c.lua".into()], b"hi".to_vec(), false)
        .unwrap();
    root.insert(&["a".into()], Vec::new(), true).unwrap();

    assert_eq!(root.file("a/b/c.lua", false), Some(&b"hi"[..]));
    assert!(root.get("a").unwrap().as_folder().is_some());
}

#[test]
fn insert_rejects_a_file_used_as_a_directory() {
    let mut root = folder("", Vec::new());
    root.insert(&["a".into()], b"file".to_vec(), false).unwrap();
    assert!(root.insert(&["a".into(), "b".into()], b"x".to_vec(), false).is_err());
}

#[test]
fn insert_keeps_the_last_duplicate_file() {
    let mut root = folder("", Vec::new());
    root.insert(&["a.lua".into()], b"first".to_vec(), false).unwrap();
    root.insert(&["a.lua".into()], b"second".to_vec(), false).unwrap();

    assert_eq!(root.file("a.lua", false), Some(&b"second"[..]));
    assert_eq!(root.as_folder().unwrap().len(), 1);
}

#[test]
fn sort_puts_folders_before_files() {
    let mut root = folder("", vec![file("z.lua", ""), folder("a", Vec::new())]);
    root.sort();

    let names: Vec<_> = root.as_folder().unwrap().iter().map(ScriptStruct::name).collect();
    assert_eq!(names, ["a", "z.lua"]);
}

#[test]
fn unwraps_a_single_top_level_directory() {
    let root = folder("", vec![folder("script", vec![file("Script.toml", "[script]")])]);
    let unwrapped = Script::unwrap_archive_root(root).unwrap();
    assert!(unwrapped.file("Script.toml", false).is_some());
}

#[test]
fn keeps_the_root_when_the_manifest_is_already_there() {
    let root = folder("", vec![file("Script.toml", "[script]"), folder("folder", Vec::new())]);
    let unwrapped = Script::unwrap_archive_root(root).unwrap();
    assert!(unwrapped.file("Script.toml", false).is_some());
    assert!(unwrapped.get("folder").is_some());
}

#[test]
fn get_file_descends_into_folders() {
    let mut root = folder("", Vec::new());
    root.insert(&["folder".into(), "some_file.lua".into()], b"m".to_vec(), false)
        .unwrap();

    // Folders are matched by their exact name, files by their `.lua`-less name.
    assert!(root.get_file("folder/some_file", true).is_some());
    assert!(root.get_file("folder/some_file.lua", false).is_some());
    assert!(root.get_file("folder/some_file", false).is_none());
    assert!(root.get_file("folder", true).is_some());
    assert!(root.get_file("nope/some_file", true).is_none());
}

#[test]
fn get_file_only_strips_a_real_lua_suffix() {
    let mut root = folder("", Vec::new());
    root.insert(&["data.txt".into()], b"x".to_vec(), false).unwrap();

    // `data.txt` must not be truncated to `data`.
    assert!(root.get_file("data.txt", true).is_some());
    assert!(root.get_file("data", true).is_none());
}

#[test]
fn rejects_unsafe_paths() {
    assert!(Script::normalize_path(Path::new("../escape.lua")).is_err());
    assert!(Script::normalize_path(Path::new("/abs.lua")).is_err());
    assert_eq!(
        Script::normalize_path(Path::new("./a/./b.lua")).unwrap(),
        vec!["a".to_owned(), "b.lua".to_owned()]
    );
}
