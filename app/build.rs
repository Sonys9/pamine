//! Build-time code generation for the block and registry tables.
//!
//! Two inputs, two outputs:
//!
//! | input                  | output                        | generator            |
//! |------------------------|-------------------------------|----------------------|
//! | `data/blocks.json`     | `$OUT_DIR/blocks_gen.rs`      | `gen_blocks`         |
//! | `data/registries.json` | `$OUT_DIR/registries_gen.rs`  | `gen_registries`     |
//!
//! `blocks.json` holds the physics of every block (hardness, drops, states, …)
//! but no item names; `registries.json` holds every registry's `(name, id)`
//! pairs but no physics. The two are cross-checked: the `minecraft:block`
//! registry must agree with `blocks.json` on all 933 names and ids, and every
//! id in `drops` / `harvestTools` must resolve in `minecraft:item`.
//!
//! Both files live in `data/`, not `src/`: they are build input, not Rust
//! source, and keeping them out of `src/` stops them from being mistaken for
//! modules and keeps `cargo package` from shipping them as source.

use std::{env, fs, path::PathBuf};

use serde_json::Value;

#[path = "build/common.rs"]
mod common;
#[path = "build/gen_blocks.rs"]
mod gen_blocks;
#[path = "build/gen_registries.rs"]
mod gen_registries;

fn main() {
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set"));
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is set"));

    let blocks_path = manifest_dir.join("data").join("blocks.json");
    let registries_path = manifest_dir.join("data").join("registries.json");
    println!("cargo:rerun-if-changed={}", blocks_path.display());
    println!("cargo:rerun-if-changed={}", registries_path.display());

    let blocks_root: Value = read_json(&blocks_path);
    let blocks: Vec<Value> = match blocks_root {
        Value::Array(blocks) => blocks,
        _ => panic!("{} must be a JSON array", blocks_path.display()),
    };
    let registries_root: Value = read_json(&registries_path);

    let registries = gen_registries::parse(&registries_root);

    let block_registry = registries
        .iter()
        .find(|registry| registry.key == "minecraft:block")
        .expect("registries.json has no `minecraft:block` registry");
    let item_registry = registries
        .iter()
        .find(|registry| registry.key == "minecraft:item")
        .expect("registries.json has no `minecraft:item` registry");

    gen_blocks::cross_validate(&blocks, block_registry);

    let items = gen_blocks::ItemIndex::new(item_registry);

    write(
        &out_dir.join("registries_gen.rs"),
        gen_registries::generate(&registries),
    );
    write(
        &out_dir.join("blocks_gen.rs"),
        gen_blocks::generate(&blocks, &items),
    );
}

fn read_json(path: &PathBuf) -> Value {
    let raw = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    serde_json::from_str(&raw).unwrap_or_else(|e| panic!("failed to parse {}: {e}", path.display()))
}

fn write(path: &PathBuf, contents: String) {
    fs::write(path, contents).unwrap_or_else(|e| panic!("failed to write {}: {e}", path.display()));
}
