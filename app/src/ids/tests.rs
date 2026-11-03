//! Consistency tests for the generated block and registry tables.
//!
//! `super` is `crate::ids`, which re-exports everything from the two generated
//! files, so the tables are in scope without extra imports.

use super::*;

#[test]
fn blocks_are_consistent() {
    assert_eq!(BLOCKS.len(), BLOCK_COUNT);
    for (index, block) in BLOCKS.iter().enumerate() {
        assert_eq!(block.id as usize, index, "ids must be contiguous");
        assert!(block.min_state_id <= block.max_state_id);
        assert!(block.default_state >= block.min_state_id);
        assert!(block.default_state <= block.max_state_id);
        assert_eq!(
            block.state_count(),
            block.max_state_id - block.min_state_id + 1
        );
    }
}

#[test]
fn block_lookups_work() {
    let stone = Block::by_name("stone").expect("stone exists");
    assert_eq!(stone.id, 1);
    assert_eq!(Block::by_id(1).map(|b| b.name), Some("stone"));
    assert_eq!(Block::by_id(BLOCK_COUNT as u16), None);
    assert_eq!(stone.bounding_box, BoundingBox::Block);
}

#[test]
fn drops_resolve_to_items() {
    let stone = Block::by_name("stone").expect("stone exists");
    assert_eq!(
        stone.drops,
        &[Item {
            id: 22,
            name: "cobblestone"
        }]
    );

    let diamond_ore = Block::by_name("diamond_ore").expect("diamond_ore exists");
    assert_eq!(
        diamond_ore.drops,
        &[Item {
            id: 722,
            name: "diamond"
        }]
    );

    let air = Block::by_name("air").expect("air exists");
    assert!(air.drops.is_empty());
}

#[test]
fn harvest_tools_resolve_to_items() {
    let stone = Block::by_name("stone").expect("stone exists");
    let pickaxe = stone
        .harvest_tools
        .iter()
        .find(|tool| tool.item.name == "wooden_pickaxe")
        .expect("stone is mineable with a wooden pickaxe");
    assert_eq!(pickaxe.item.id, 737);
    assert!(pickaxe.can_harvest);

    // Sorted by item id, not by the JSON object's lexicographic key order.
    let ids: Vec<u16> = stone.harvest_tools.iter().map(|t| t.item.id).collect();
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    assert_eq!(ids, sorted);
}

#[test]
fn states_are_typed() {
    let grass = Block::by_name("grass_block").expect("grass_block exists");
    assert_eq!(grass.states.len(), 1);
    assert_eq!(grass.states[0].kind, StateKind::Bool);
    assert_eq!(grass.states[0].values, None);

    let log = Block::by_name("oak_log").expect("oak_log exists");
    let axis = log
        .states
        .iter()
        .find(|s| s.name == "axis")
        .expect("axis state");
    assert_eq!(axis.kind, StateKind::Enum);
    assert_eq!(axis.values, Some(StateValue::Enum(&["x", "y", "z"])));

    let sapling = Block::by_name("oak_sapling").expect("oak_sapling exists");
    let stage = sapling
        .states
        .iter()
        .find(|s| s.name == "stage")
        .expect("stage state");
    assert_eq!(stage.kind, StateKind::Int);
    assert_eq!(stage.values, Some(StateValue::Int(&[0, 1])));
}

#[test]
fn materials_are_split() {
    let grass = Block::by_name("grass_block").expect("grass_block exists");
    assert!(grass.material.mineable_shovel);
    assert!(!grass.material.mineable_pickaxe);

    let vine = Block::by_name("vine").expect("vine exists");
    assert!(vine.material.plant);
    assert!(vine.material.vine_or_glow_lichen);
    assert!(vine.material.mineable_axe);
}

#[test]
fn registries_are_dense_and_sorted() {
    assert_eq!(REGISTRIES.len(), 63);
    for (index, registry) in REGISTRIES.iter().enumerate() {
        assert_eq!(registry.id as usize, index, "registry ids must be dense");
        let ids: Vec<u16> = registry.entries.iter().map(|e| e.id).collect();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        assert_eq!(ids, sorted, "{} is not sorted by id", registry.name);
    }
}

#[test]
fn registry_lookups_work() {
    let entity = registry("entity_type").expect("entity_type exists");
    assert_eq!(entity.name, "minecraft:entity_type");
    assert_eq!(entity.default, Some("pig"));
    assert_eq!(entity.len(), 118);
    assert_eq!(entity.name_of(0), Some("allay"));
    assert_eq!(entity.name_of(116), Some("player"));
    assert_eq!(entity.name_of(117), Some("fishing_bobber"));
    assert_eq!(entity.id_of("creeper"), Some(15));
    assert_eq!(entity.by_id(118), None);

    // The namespace is optional on lookup.
    assert!(registry("minecraft:entity_type").is_some());
    assert!(registry("nope").is_none());
}

#[test]
fn mob_effect_is_the_only_sparse_registry() {
    let sparse: Vec<&str> = REGISTRIES
        .iter()
        .filter(|registry| {
            registry
                .entries
                .iter()
                .enumerate()
                .any(|(index, entry)| entry.id as usize != index)
        })
        .map(|registry| registry.name)
        .collect();
    assert_eq!(sparse, ["minecraft:mob_effect"]);

    let effects = registry("mob_effect").expect("mob_effect exists");
    assert_eq!(effects.entries.first().map(|e| e.id), Some(1));
    assert_eq!(effects.by_id(0), None);
}

#[test]
fn items_are_dense() {
    assert_eq!(ITEMS.len(), 1152);
    for (index, item) in ITEMS.iter().enumerate() {
        assert_eq!(item.id as usize, index);
    }
    assert_eq!(ITEMS[0].name, "air");
}

#[test]
fn block_registry_matches_blocks_json() {
    let registry = registry("block").expect("block registry exists");
    assert_eq!(registry.len(), BLOCKS.len());
    for block in BLOCKS.iter() {
        assert_eq!(registry.id_of(block.name), Some(block.id));
    }
}
