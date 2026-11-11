//! End-to-end tests against the bundled `test_script.tar`.
//!
//! Unit tests for the archive internals live in [`crate::scripts::loader`].
//!
//! The archive and the sources it was built from both live in
//! `app/tests/scripts/`, next to the other test fixtures, rather than inside
//! `src/`. Rebuild it with `app/tests/scripts/build.sh` after editing a source.

use super::Script;

fn test_script() -> Script {
    Script::parse(include_bytes!("../../tests/scripts/test_script.tar").to_vec())
        .expect("the bundled test script must parse")
}

#[test]
fn reads_the_manifest() {
    let script = test_script();
    assert_eq!(script.name, "Test script");
    assert_eq!(script.description, "Just a test script");
    assert_eq!(script.engine_version, 1);
    assert_eq!(script.main_file, "test_script.lua");
    assert_eq!(
        script.uuid.to_string(),
        "dafe77bb-e1d5-4560-b473-288676192212"
    );
}

#[test]
fn builds_the_file_tree() {
    let script = test_script();
    assert!(script.files.file("Script.toml", false).is_some());
    assert!(script.files.file("test_script.lua", false).is_some());
    assert!(script.files.file("folder/data.txt", false).is_some());
    assert!(script.files.file("folder/data2.txt", false).is_some());
    assert!(script.files.file("folder/folder/data.txt", false).is_some());
    assert!(script.files.file("folder2/data.txt", false).is_some());
    // An empty directory is not a file.
    assert!(script.files.file("folder2/folder", false).is_none());
    assert!(script.files.file("nope.lua", false).is_none());
}

#[test]
fn resolves_lua_modules_in_nested_folders() {
    let script = test_script();
    // `require("folder.some_file")` becomes `folder/some_file`.
    assert!(script.files.file("folder/some_file", true).is_some());
    assert!(script.files.file("required_file", true).is_some());
    assert!(script.files.file("folder/some_file.lua", false).is_some());
}

#[test]
fn exposes_the_main_source() {
    let script = test_script();
    assert!(script.main_code().unwrap().contains("engine:register"));
}

/// `changeable` is the one `Injection` field a script may write.
///
/// Without a setter mlua installs no `__newindex`, so the assignment fails with
/// "attempt to index ... (a userdata value)" and the flag silently stays as it
/// was. The other fields must stay read-only.
#[test]
fn lua_can_flip_the_changeable_flag() {
    use mlua::prelude::*;

    let lua = Lua::new();
    lua.globals()
        .set("inject", crate::connections::Injection::empty(true))
        .unwrap();

    assert!(lua.load("return inject.changeable").eval::<bool>().unwrap());

    lua.load("inject.changeable = false").exec().unwrap();
    assert!(!lua.load("return inject.changeable").eval::<bool>().unwrap());

    // The write reached the Rust value, not just a Lua-side shadow.
    let userdata: LuaAnyUserData = lua.globals().get("inject").unwrap();
    assert!(!userdata.borrow::<crate::connections::Injection>().unwrap().changeable);

    // Read-only fields still reject an assignment.
    assert!(lua.load("inject.depth = 5").exec().is_err());
    assert!(lua.load("inject.len = 5").exec().is_err());
}
