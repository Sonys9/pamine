//! Generates `$OUT_DIR/blocks_gen.rs` from `data/blocks.json`.
//!
//! The JSON is a 655 KB array of 933 block definitions. Parsing it at runtime
//! (the old `IdParser`) meant a tokio file read on every start and a `String`
//! buffer that had to be re-parsed by hand. Instead we turn it into plain
//! `static` data at compile time, so the binary just links the tables.
//!
//! `drops` and `harvestTools` hold *item* ids, so they are resolved against the
//! `minecraft:item` registry and emitted as `Item` values rather than bare
//! numbers.

use std::collections::BTreeSet;

use serde_json::Value;

use crate::common::{
    array, as_bool, as_f64, as_str, as_u64, f32_lit, f64_lit, ident, lit, upper_ident,
};
use crate::gen_registries::RegistryData;

/// Item id -> item name, used to resolve drops and harvest tools.
pub struct ItemIndex<'a> {
    names: Vec<&'a str>,
}

impl<'a> ItemIndex<'a> {
    pub fn new(items: &'a RegistryData) -> Self {
        let max = items
            .entries
            .iter()
            .map(|(_, id)| *id as usize)
            .max()
            .unwrap_or(0);
        let mut names = vec![""; max + 1];
        for (name, id) in &items.entries {
            names[*id as usize] = name.as_str();
        }
        Self { names }
    }

    fn resolve(&self, id: u16, context: &str) -> String {
        match self.names.get(id as usize) {
            Some(name) if !name.is_empty() => {
                format!("Item {{ id: {id}, name: {} }}", lit(name))
            }
            _ => panic!("{context}: item id {id} is not in the `minecraft:item` registry"),
        }
    }
}

/// Fails the build when `blocks.json` and the `minecraft:block` registry
/// disagree. They are two independent dumps of the same data, so a mismatch
/// means one of them was updated without the other.
pub fn cross_validate(blocks: &[Value], block_registry: &RegistryData) {
    let mut from_json: Vec<(&str, u16)> = blocks
        .iter()
        .map(|block| {
            let name = as_str(block, "name", "block");
            let id = as_u64(block, "id", &format!("block `{name}`"));
            let id = u16::try_from(id)
                .unwrap_or_else(|_| panic!("block `{name}`: id {id} does not fit in u16"));
            (name, id)
        })
        .collect();
    from_json.sort_unstable();

    let mut from_registry: Vec<(&str, u16)> = block_registry
        .entries
        .iter()
        .map(|(name, id)| (name.as_str(), *id))
        .collect();
    from_registry.sort_unstable();

    if from_json != from_registry {
        let json_names: BTreeSet<&str> = from_json.iter().map(|(name, _)| *name).collect();
        let registry_names: BTreeSet<&str> = from_registry.iter().map(|(name, _)| *name).collect();

        let only_json: Vec<&&str> = json_names.difference(&registry_names).collect();
        let only_registry: Vec<&&str> = registry_names.difference(&json_names).collect();

        let mut detail = String::new();
        if !only_json.is_empty() {
            detail.push_str(&format!("\n  only in blocks.json: {only_json:?}"));
        }
        if !only_registry.is_empty() {
            detail.push_str(&format!("\n  only in registries.json: {only_registry:?}"));
        }
        for (json, registry) in from_json.iter().zip(from_registry.iter()) {
            if json != registry {
                detail.push_str(&format!(
                    "\n  first id mismatch: blocks.json has {json:?}, registries.json has {registry:?}"
                ));
                break;
            }
        }

        panic!(
            "blocks.json and the `minecraft:block` registry disagree \
             ({} vs {} entries){detail}",
            from_json.len(),
            from_registry.len()
        );
    }
}

pub fn generate(blocks: &[Value], items: &ItemIndex<'_>) -> String {
    // `material` is a `;`-separated tag list, e.g. `"gourd;mineable/axe"`.
    // Collect every tag and every distinct combination so we can emit one
    // `Material` const per combination instead of repeating 11 bools 933 times.
    let mut material_tags: BTreeSet<String> = BTreeSet::new();
    let mut material_values: BTreeSet<String> = BTreeSet::new();
    for block in blocks {
        let material = as_str(block, "material", "block");
        material_values.insert(material.to_string());
        for tag in material.split(';') {
            material_tags.insert(tag.to_string());
        }
    }

    let mut out = String::with_capacity(1 << 20);
    out.push_str(HEADER);

    out.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]\npub struct Material {\n");
    for tag in &material_tags {
        out.push_str(&format!("    pub {}: bool,\n", ident(tag)));
    }
    out.push_str("}\n\n");

    for material in &material_values {
        out.push_str(&format!(
            "pub const {}: Material = Material {{\n",
            const_name(material)
        ));
        for tag in &material_tags {
            let present = material.split(';').any(|t| t == tag);
            out.push_str(&format!("    {}: {},\n", ident(tag), present));
        }
        out.push_str("};\n\n");
    }

    out.push_str(BLOCK_STRUCT);

    out.push_str(&format!(
        "pub const BLOCK_COUNT: usize = {};\n\n",
        blocks.len()
    ));
    out.push_str(&format!(
        "pub static BLOCKS: [Block; {}] = [\n",
        blocks.len()
    ));
    for block in blocks {
        out.push_str(&gen_block(block, items));
    }
    out.push_str("];\n");
    out.push_str(HELPERS);
    out
}

fn gen_block(block: &Value, items: &ItemIndex<'_>) -> String {
    let name = as_str(block, "name", "block");
    let context = format!("block `{name}`");

    let mut s = String::with_capacity(1024);
    s.push_str("    Block {\n");
    s.push_str(&format!("        id: {},\n", as_u64(block, "id", &context)));
    s.push_str(&format!("        name: {},\n", lit(name)));
    s.push_str(&format!(
        "        display_name: {},\n",
        lit(as_str(block, "displayName", &context))
    ));
    s.push_str(&format!(
        "        hardness: {},\n",
        f32_lit(as_f64(block, "hardness", &context))
    ));
    s.push_str(&format!(
        "        resistance: {},\n",
        f64_lit(as_f64(block, "resistance", &context))
    ));
    s.push_str(&format!(
        "        stack_size: {},\n",
        as_u64(block, "stackSize", &context)
    ));
    s.push_str(&format!(
        "        diggable: {},\n",
        as_bool(block, "diggable", &context)
    ));
    s.push_str(&format!(
        "        material: {},\n",
        const_name(as_str(block, "material", &context))
    ));
    s.push_str(&format!(
        "        transparent: {},\n",
        as_bool(block, "transparent", &context)
    ));
    s.push_str(&format!(
        "        emit_light: {},\n",
        as_u64(block, "emitLight", &context)
    ));
    s.push_str(&format!(
        "        filter_light: {},\n",
        as_u64(block, "filterLight", &context)
    ));
    s.push_str(&format!(
        "        default_state: {},\n",
        as_u64(block, "defaultState", &context)
    ));
    s.push_str(&format!(
        "        min_state_id: {},\n",
        as_u64(block, "minStateId", &context)
    ));
    s.push_str(&format!(
        "        max_state_id: {},\n",
        as_u64(block, "maxStateId", &context)
    ));
    s.push_str(&format!(
        "        states: {},\n",
        gen_states(block, &context)
    ));
    s.push_str(&format!(
        "        harvest_tools: {},\n",
        gen_tools(block, items, &context)
    ));
    s.push_str(&format!(
        "        drops: {},\n",
        gen_drops(block, items, &context)
    ));
    s.push_str(&format!(
        "        bounding_box: {},\n",
        gen_bbox(block, &context)
    ));
    s.push_str("    },\n");
    s
}

fn gen_states(block: &Value, context: &str) -> String {
    let states = array(block, "states");
    if states.is_empty() {
        return "&[]".to_string();
    }

    let mut parts = Vec::with_capacity(states.len());
    for state in states {
        let name = lit(as_str(state, "name", context));
        let (kind, values) = match as_str(state, "type", context) {
            "bool" => ("StateKind::Bool", "None".to_string()),
            "int" => {
                // The game serialises int state values as strings: ["0", "1"].
                let nums: Vec<String> = array(state, "values")
                    .iter()
                    .map(|value| {
                        let raw = value.as_str().unwrap_or_else(|| {
                            panic!("{context}: int state value is not a string")
                        });
                        let number: i32 = raw.parse().unwrap_or_else(|e| {
                            panic!("{context}: int state value `{raw}` is not an i32: {e}")
                        });
                        number.to_string()
                    })
                    .collect();
                (
                    "StateKind::Int",
                    format!("Some(StateValue::Int(&[{}]))", nums.join(", ")),
                )
            }
            "enum" => {
                let strings: Vec<String> = array(state, "values")
                    .iter()
                    .map(|value| {
                        lit(value.as_str().unwrap_or_else(|| {
                            panic!("{context}: enum state value is not a string")
                        }))
                    })
                    .collect();
                (
                    "StateKind::Enum",
                    format!("Some(StateValue::Enum(&[{}]))", strings.join(", ")),
                )
            }
            other => panic!("{context}: unknown state type `{other}`"),
        };
        parts.push(format!(
            "State {{ name: {name}, kind: {kind}, values: {values} }}"
        ));
    }
    format!("&[{}]", parts.join(", "))
}

fn gen_tools(block: &Value, items: &ItemIndex<'_>, context: &str) -> String {
    // `harvestTools` is a JSON *object*: {"737": true, "742": true, ...}.
    // serde_json hands us a BTreeMap, i.e. lexicographic order, so "1000"
    // would sort before "737". Sort numerically instead.
    let Some(tools) = block.get("harvestTools").and_then(Value::as_object) else {
        return "&[]".to_string();
    };

    let mut entries: Vec<(u16, bool)> = tools
        .iter()
        .map(|(key, value)| {
            let id: u16 = key.parse().unwrap_or_else(|e| {
                panic!("{context}: harvest tool key `{key}` is not a u16: {e}")
            });
            let can_harvest = value
                .as_bool()
                .unwrap_or_else(|| panic!("{context}: harvest tool value is not a bool"));
            (id, can_harvest)
        })
        .collect();
    entries.sort_unstable_by_key(|(id, _)| *id);

    let parts: Vec<String> = entries
        .iter()
        .map(|(id, can_harvest)| {
            format!(
                "HarvestTool {{ item: {}, can_harvest: {can_harvest} }}",
                items.resolve(*id, context)
            )
        })
        .collect();
    format!("&[{}]", parts.join(", "))
}

fn gen_drops(block: &Value, items: &ItemIndex<'_>, context: &str) -> String {
    let drops = array(block, "drops");
    if drops.is_empty() {
        return "&[]".to_string();
    }
    let parts: Vec<String> = drops
        .iter()
        .map(|drop| {
            let id = drop
                .as_u64()
                .unwrap_or_else(|| panic!("{context}: drop is not an unsigned integer"));
            let id = u16::try_from(id)
                .unwrap_or_else(|_| panic!("{context}: drop id {id} does not fit in u16"));
            items.resolve(id, context)
        })
        .collect();
    format!("&[{}]", parts.join(", "))
}

fn gen_bbox(block: &Value, context: &str) -> String {
    match as_str(block, "boundingBox", context) {
        "block" => "BoundingBox::Block".to_string(),
        "empty" => "BoundingBox::Empty".to_string(),
        other => panic!("{context}: unknown boundingBox `{other}`"),
    }
}

fn const_name(material: &str) -> String {
    format!("MATERIAL_{}", upper_ident(material))
}

const HEADER: &str = r##"// @generated by build.rs from data/blocks.json. Do not edit by hand.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateKind {
    Bool,
    Int,
    Enum,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundingBox {
    Block,
    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateValue {
    Bool,
    Int(&'static [i32]),
    Enum(&'static [&'static str]),
}

impl StateValue {
    /// Number of distinct values this state can take.
    pub fn len(&self) -> usize {
        match self {
            StateValue::Bool => 2,
            StateValue::Int(values) => values.len(),
            StateValue::Enum(values) => values.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct State {
    pub name: &'static str,
    pub kind: StateKind,
    pub values: Option<StateValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HarvestTool {
    pub item: Item,
    pub can_harvest: bool,
}

"##;

const BLOCK_STRUCT: &str = r##"#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Block {
    pub id: u16,
    pub name: &'static str,
    pub display_name: &'static str,
    pub hardness: f32,
    pub resistance: f64,
    pub stack_size: u8,
    pub diggable: bool,
    pub material: Material,
    pub transparent: bool,
    pub emit_light: u8,
    pub filter_light: u8,
    pub default_state: u16,
    pub min_state_id: u16,
    pub max_state_id: u16,
    pub states: &'static [State],
    pub harvest_tools: &'static [HarvestTool],
    pub drops: &'static [Item],
    pub bounding_box: BoundingBox,
}

"##;

const HELPERS: &str = r##"
impl Block {
    /// Looks a block up by its numeric id. Ids are contiguous `0..BLOCK_COUNT`,
    /// so this is O(1).
    pub fn by_id(id: u16) -> Option<&'static Block> {
        BLOCKS.get(id as usize).filter(|block| block.id == id)
    }

    /// Looks a block up by its registry name, e.g. `"grass_block"`.
    pub fn by_name(name: &str) -> Option<&'static Block> {
        BLOCKS.iter().find(|block| block.name == name)
    }

    /// Number of block states this block occupies in the global state id space.
    pub fn state_count(&self) -> u16 {
        self.max_state_id - self.min_state_id + 1
    }
}

impl TryFrom<u16> for &'static Block {
    type Error = UnknownId;

    fn try_from(id: u16) -> Result<Self, Self::Error> {
        Block::by_id(id).ok_or(UnknownId(id))
    }
}

impl<'a> TryFrom<&'a str> for &'static Block {
    type Error = UnknownName<'a>;

    fn try_from(name: &'a str) -> Result<Self, Self::Error> {
        Block::by_name(name).ok_or(UnknownName(name))
    }
}
"##;
