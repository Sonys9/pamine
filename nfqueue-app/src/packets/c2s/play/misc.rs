//! Entity interaction, recipes, advancements and trade selection.

use pcore::{
    Decode,
    decoder::{Uuid, VarInt},
};

#[derive(Decode, Debug, Clone)]
pub struct QueryEntityTag {
    pub transaction_id: VarInt,
    pub entity_id: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct Interact {
    pub entity_id: VarInt,
    pub kind: VarInt,
    pub target_x: Option<f32>,
    pub target_y: Option<f32>,
    pub target_z: Option<f32>,
    pub hand: Option<VarInt>,
    pub sneaking: bool,
}

#[derive(Decode, Debug, Clone)]
pub struct SwingArm {
    pub hand: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct TeleportToEntity {
    pub target: Uuid,
}

#[derive(Decode, Debug, Clone)]
pub struct PickItem {
    pub slot: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct PlaceRecipe<'a> {
    pub window_id: u8,
    pub recipe: &'a str,
    pub make_all: bool,
}

#[derive(Decode, Debug, Clone)]
pub struct ChangeRecipeBookSettings {
    pub book_id: VarInt,
    pub book_open: bool,
    pub filter_active: bool,
}

#[derive(Decode, Debug, Clone)]
pub struct SetSeenRecipe<'a> {
    pub recipe_id: &'a str,
}

#[derive(Decode, Debug, Clone)]
pub struct RenameItem<'a> {
    pub item_name: &'a str,
}

#[derive(Decode, Debug, Clone)]
pub struct SeenAdvancements<'a> {
    pub action: VarInt,
    pub tab_id: Option<&'a str>,
}

#[derive(Decode, Debug, Clone)]
pub struct SelectTrade {
    pub selected_slot: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct SetBeaconEffect {
    pub has_primary_effect: bool,
    pub primary_effect: Option<VarInt>,
    pub has_secondary_effect: bool,
    pub secondary_effect: Option<VarInt>,
}

#[derive(Decode, Debug, Clone)]
pub struct SetHeldItem {
    pub slot: i16,
}
