//! Entity interaction, recipes, advancements and trade selection.

use pcore::{
    Decode,
    decoder::{Decoder, Uuid, VarInt},
    errors::Error,
};

#[derive(Decode, Debug, Clone)]
pub struct QueryEntityTag {
    pub transaction_id: VarInt,
    pub entity_id: VarInt,
}

/// `Interact` (0x10).
///
/// Hand-written `Decode`: the target coordinates are only present for
/// `type == 2` (interact at), and `hand` only for `type == 0` or `2`.
#[derive(Debug, Clone)]
pub struct Interact {
    pub entity_id: VarInt,
    /// 0 = interact, 1 = attack, 2 = interact at.
    pub kind: VarInt,
    pub target_x: Option<f32>,
    pub target_y: Option<f32>,
    pub target_z: Option<f32>,
    /// 0 = main hand, 1 = off hand.
    pub hand: Option<VarInt>,
    pub sneaking: bool,
}

impl<'a> Decode<'a> for Interact {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        const INTERACT: i32 = 0;
        const INTERACT_AT: i32 = 2;

        let entity_id = decoder.varint()?;
        let kind = decoder.varint()?;
        let kind_id = kind.get();

        let (target_x, target_y, target_z) = if kind_id == INTERACT_AT {
            (
                Some(decoder.f32()?),
                Some(decoder.f32()?),
                Some(decoder.f32()?),
            )
        } else {
            (None, None, None)
        };
        let hand = if kind_id == INTERACT || kind_id == INTERACT_AT {
            Some(decoder.varint()?)
        } else {
            None
        };
        let sneaking = decoder.bool()?;

        Ok(Self {
            entity_id,
            kind,
            target_x,
            target_y,
            target_z,
            hand,
            sneaking,
        })
    }
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
    pub window_id: i8,
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

/// `Seen Advancements` (0x25).
///
/// Hand-written `Decode`: `tab_id` is only present for `action == 0`
/// (opened tab).
#[derive(Debug, Clone)]
pub struct SeenAdvancements<'a> {
    /// 0 = opened tab, 1 = closed screen.
    pub action: VarInt,
    pub tab_id: Option<&'a str>,
}

impl<'a> Decode<'a> for SeenAdvancements<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let action = decoder.varint()?;
        let tab_id = if action.get() == 0 {
            Some(decoder.string()?)
        } else {
            None
        };
        Ok(Self { action, tab_id })
    }
}

#[derive(Decode, Debug, Clone)]
pub struct SelectTrade {
    pub selected_slot: VarInt,
}

/// `Set Beacon Effect` (0x27).
///
/// Hand-written `Decode`: each effect is gated by its own boolean, which
/// `Option<VarInt>` would consume a second time.
#[derive(Debug, Clone)]
pub struct SetBeaconEffect {
    pub has_primary_effect: bool,
    pub primary_effect: Option<VarInt>,
    pub has_secondary_effect: bool,
    pub secondary_effect: Option<VarInt>,
}

impl<'a> Decode<'a> for SetBeaconEffect {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let has_primary_effect = decoder.bool()?;
        let primary_effect = if has_primary_effect {
            Some(decoder.varint()?)
        } else {
            None
        };
        let has_secondary_effect = decoder.bool()?;
        let secondary_effect = if has_secondary_effect {
            Some(decoder.varint()?)
        } else {
            None
        };
        Ok(Self {
            has_primary_effect,
            primary_effect,
            has_secondary_effect,
            secondary_effect,
        })
    }
}

#[derive(Decode, Debug, Clone)]
pub struct SetHeldItem {
    pub slot: i16,
}
