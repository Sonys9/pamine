//! Generates `$OUT_DIR/registries_gen.rs` from `data/registries.json`.
//!
//! The report is a flat map of 63 registries, each of the shape
//! `{ "default"?: name, "protocol_id": u8, "entries": { name: { "protocol_id": u16 } } }`.
//! Every entry carries exactly one field, so the generated tables are just
//! `(name, id)` pairs sorted by id.

use serde_json::Value;

use crate::common::{as_u64, lit, strip_namespace, upper_ident};

/// One registry, already validated and sorted by protocol id.
pub struct RegistryData {
    /// Full key, e.g. `minecraft:entity_type`.
    pub key: String,
    /// The registry's own protocol id, `0..=62`.
    pub id: u8,
    /// Default entry name with the namespace stripped, if the registry has one.
    pub default: Option<String>,
    /// `(name without namespace, protocol id)`, sorted by id.
    pub entries: Vec<(String, u16)>,
}

pub fn parse(root: &Value) -> Vec<RegistryData> {
    let object = root
        .as_object()
        .expect("registries.json must be a JSON object");

    let mut registries = Vec::with_capacity(object.len());

    for (key, value) in object {
        let context = format!("registry `{key}`");

        let raw_id = as_u64(value, "protocol_id", &context);
        let id = u8::try_from(raw_id)
            .unwrap_or_else(|_| panic!("{context}: id {raw_id} does not fit in u8"));

        let default = value
            .get("default")
            .and_then(Value::as_str)
            .map(|name| strip_namespace(name).to_string());

        let entries_object = value
            .get("entries")
            .and_then(Value::as_object)
            .unwrap_or_else(|| panic!("{context}: missing `entries` object"));

        let mut entries: Vec<(String, u16)> = entries_object
            .iter()
            .map(|(name, entry)| {
                let raw = as_u64(entry, "protocol_id", &format!("{context} entry `{name}`"));
                let entry_id = u16::try_from(raw).unwrap_or_else(|_| {
                    panic!("{context} entry `{name}`: id {raw} does not fit in u16")
                });
                (strip_namespace(name).to_string(), entry_id)
            })
            .collect();

        // `by_id` binary-searches, so the slice must be ordered by id.
        entries.sort_unstable_by_key(|(_, id)| *id);

        // Duplicate ids would make `by_id` ambiguous.
        for pair in entries.windows(2) {
            if pair[0].1 == pair[1].1 {
                panic!(
                    "{context}: `{}` and `{}` share protocol id {}",
                    pair[0].0, pair[1].0, pair[0].1
                );
            }
        }

        // Duplicate names would make `by_name` ambiguous.
        let mut names: Vec<&str> = entries.iter().map(|(name, _)| name.as_str()).collect();
        names.sort_unstable();
        for pair in names.windows(2) {
            if pair[0] == pair[1] {
                panic!("{context}: duplicate entry name `{}`", pair[0]);
            }
        }

        registries.push(RegistryData {
            key: key.clone(),
            id,
            default,
            entries,
        });
    }

    registries.sort_unstable_by_key(|registry| registry.id);

    // Registry ids are dense in 1.19.2. A gap means the report was produced by
    // a different version than the one the rest of the tables were built for.
    for (index, registry) in registries.iter().enumerate() {
        if registry.id as usize != index {
            panic!(
                "registry ids are not dense: expected {index} at position {index}, found {} (`{}`)",
                registry.id, registry.key
            );
        }
    }

    registries
}

pub fn generate(registries: &[RegistryData]) -> String {
    let mut out = String::with_capacity(1 << 19);
    out.push_str(HEADER);

    // One named static per registry, so `REGISTRIES` can borrow them.
    for registry in registries {
        out.push_str(&format!(
            "pub static {}: Registry = Registry {{\n    id: {},\n    name: {},\n    default: {},\n    entries: &[\n",
            static_name(&registry.key),
            registry.id,
            lit(&registry.key),
            match &registry.default {
                Some(name) => format!("Some({})", lit(name)),
                None => "None".to_string(),
            },
        ));
        for (name, id) in &registry.entries {
            out.push_str(&format!(
                "        RegistryEntry {{ name: {}, id: {} }},\n",
                lit(name),
                id
            ));
        }
        out.push_str("    ],\n};\n\n");
    }

    out.push_str(&format!(
        "pub static REGISTRIES: [&Registry; {}] = [\n",
        registries.len()
    ));
    for registry in registries {
        out.push_str(&format!("    &{},\n", static_name(&registry.key)));
    }
    out.push_str("];\n\n");

    // Short aliases: `BLOCK_REGISTRY`, `ENTITY_TYPE_REGISTRY`, ...
    for registry in registries {
        out.push_str(&format!(
            "pub static {}: &Registry = &{};\n",
            alias_name(&registry.key),
            static_name(&registry.key)
        ));
    }
    out.push('\n');

    // `ITEMS` is the one registry that gets a dedicated domain type, because
    // block drops and harvest tools reference it by id.
    if let Some(items) = registries.iter().find(|r| r.key == "minecraft:item") {
        out.push_str(&format!(
            "/// Every item, sorted by id. Item ids are dense `0..ITEMS.len()`.\npub static ITEMS: [Item; {}] = [\n",
            items.entries.len()
        ));
        for (name, id) in &items.entries {
            out.push_str(&format!(
                "    Item {{ id: {}, name: {} }},\n",
                id,
                lit(name)
            ));
        }
        out.push_str("];\n");
    }

    out
}

fn static_name(key: &str) -> String {
    format!("REGISTRY_{}", upper_ident(strip_namespace(key)))
}

fn alias_name(key: &str) -> String {
    format!("{}_REGISTRY", upper_ident(strip_namespace(key)))
}

const HEADER: &str = r##"// @generated by build.rs from data/registries.json. Do not edit by hand.

/// One `(name, protocol id)` pair inside a registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegistryEntry {
    /// Name with the `minecraft:` namespace stripped, e.g. `pig`.
    pub name: &'static str,
    pub id: u16,
}

/// A single registry from `registries.json`.
#[derive(Debug, Clone, Copy)]
pub struct Registry {
    /// The registry's own protocol id, `0..=62`.
    pub id: u8,
    /// Full key, e.g. `minecraft:entity_type`.
    pub name: &'static str,
    /// Default entry name, if the registry declares one. Only 12 of the 63 do.
    pub default: Option<&'static str>,
    /// Entries sorted by id. Ids are dense in every registry except
    /// `minecraft:mob_effect`, which starts at 1.
    pub entries: &'static [RegistryEntry],
}

impl Registry {
    /// Looks an entry up by protocol id. O(log n).
    pub fn by_id(&self, id: u16) -> Option<&'static RegistryEntry> {
        self.entries
            .binary_search_by_key(&id, |entry| entry.id)
            .ok()
            .and_then(|index| self.entries.get(index))
    }

    /// Looks an entry up by name. O(n) — entries are ordered by id, not name.
    pub fn by_name(&self, name: &str) -> Option<&'static RegistryEntry> {
        self.entries.iter().find(|entry| entry.name == name)
    }

    pub fn name_of(&self, id: u16) -> Option<&'static str> {
        self.by_id(id).map(|entry| entry.name)
    }

    pub fn id_of(&self, name: &str) -> Option<u16> {
        self.by_name(name).map(|entry| entry.id)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Looks a registry up by key. Accepts the name with or without the
/// `minecraft:` namespace.
pub fn registry(name: &str) -> Option<&'static Registry> {
    let wanted = name.strip_prefix("minecraft:").unwrap_or(name);
    REGISTRIES.iter().copied().find(|registry| {
        registry.name.strip_prefix("minecraft:").unwrap_or(registry.name) == wanted
    })
}

/// An item, as referenced by block drops and harvest tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Item {
    pub id: u16,
    pub name: &'static str,
}

"##;
