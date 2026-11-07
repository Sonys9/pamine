//! End-to-end tests against the bundled `test_script.tar`.
//!
//! Unit tests for the archive internals live in [`crate::scripts::loader`].

use super::Script;

fn test_script() -> Script {
    Script::parse(include_bytes!("test_script.tar").to_vec())
        .expect("the bundled test script must parse")
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
