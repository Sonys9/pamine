use std::path::Path;

use super::{Script, tree::ScriptStruct};

fn test_script() -> Script {
    Script::parse(include_bytes!("test_script.tar").to_vec())
        .expect("the bundled test script must parse")
}

fn folder(name: &str, content: Vec<ScriptStruct>) -> ScriptStruct {
    ScriptStruct::Folder { name: name.to_owned(), content }
}

fn file(name: &str, content: &str) -> ScriptStruct {
    ScriptStruct::File { name: name.to_owned(), content: content.as_bytes().to_vec() }
}

#[test]
fn reads_the_manifest() {
    let script = test_script();
    assert_eq!(script.name, "Test script");
    assert_eq!(script.description, "Just a test script");
    assert_eq!(script.engine_version, 1);
    assert_eq!(script.main_file, "test_script.lua");
    assert_eq!(script.uuid.to_string(), "dafe77bb-e1d5-4560-b473-288676192212");
}

#[test]
fn builds_the_file_tree() {
    let script = test_script();
    assert!(script.files.file("Script.toml").is_some());
    assert!(script.files.file("test_script.lua").is_some());
    assert!(script.files.file("folder/data.txt").is_some());
    assert!(script.files.file("folder/data2.txt").is_some());
    assert!(script.files.file("folder/folder/data.txt").is_some());
    assert!(script.files.file("folder2/data.txt").is_some());
    // An empty directory is not a file.
    assert!(script.files.file("folder2/folder").is_none());
    assert!(script.files.file("nope.lua").is_none());
}

#[test]
fn exposes_the_main_source() {
    let script = test_script();
    assert!(script.main_code().unwrap().contains("engine:register"));
}

#[test]
fn insert_is_order_independent() {
    // A file listed before its parent directory must still end up nested.
    let mut root = folder("", Vec::new());
    root.insert(&["a".into(), "b".into(), "c.lua".into()], b"hi".to_vec(), false)
        .unwrap();
    root.insert(&["a".into()], Vec::new(), true).unwrap();

    assert_eq!(root.file("a/b/c.lua"), Some(&b"hi"[..]));
    assert!(root.get("a").unwrap().as_folder().is_some());
}

#[test]
fn insert_rejects_a_file_used_as_a_directory() {
    let mut root = folder("", Vec::new());
    root.insert(&["a".into()], b"file".to_vec(), false).unwrap();
    assert!(root.insert(&["a".into(), "b".into()], b"x".to_vec(), false).is_err());
}

#[test]
fn unwraps_a_single_top_level_directory() {
    let root = folder("", vec![folder("script", vec![file("Script.toml", "[script]")])]);
    let unwrapped = Script::unwrap_archive_root(root).unwrap();
    assert!(unwrapped.file("Script.toml").is_some());
}

#[test]
fn keeps_the_root_when_the_manifest_is_already_there() {
    let root = folder("", vec![file("Script.toml", "[script]"), folder("folder", Vec::new())]);
    let unwrapped = Script::unwrap_archive_root(root).unwrap();
    assert!(unwrapped.file("Script.toml").is_some());
    assert!(unwrapped.get("folder").is_some());
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
