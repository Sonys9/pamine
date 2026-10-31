//! Containers, slots, items and the recipe book.

use pcore::{
    Decode, Encode,
    decoder::{Decoder, Slot, VarInt},
    encoder::Encoder,
    errors::Error,
};

use crate::packets::{counted, identifiers};

#[derive(Decode, Encode, Debug, Clone)]
pub struct CloseContainer {
    pub window_id: u8,
}

#[derive(Debug, Clone)]
pub struct SetContainerContent<'a> {
    pub window_id: u8,
    pub state_id: VarInt,
    /// Every slot of the window, in order.
    pub slots: Vec<Slot<'a>>,
    /// The item held by the cursor.
    pub carried_item: Slot<'a>,
}

impl<'a> Decode<'a> for SetContainerContent<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let window_id = decoder.u8()?;
        let state_id = decoder.varint()?;

        // `count` is a VarInt, and each of the `count` slots is a full Slot.
        let count = decoder.varint()?.get();
        if count < 0 {
            return Err(Error::Read(0));
        }
        let mut slots = Vec::with_capacity(count as usize);
        for _ in 0..count {
            slots.push(Slot::decode(decoder)?);
        }
        let carried_item = Slot::decode(decoder)?;

        Ok(Self {
            window_id,
            state_id,
            slots,
            carried_item,
        })
    }
}

impl Encode for SetContainerContent<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.u8(self.window_id);
        encoder.varint(self.state_id.get());
        encoder.varint(self.slots.len() as i32);
        for slot in &self.slots {
            encoder.slot(slot);
        }
        encoder.slot(&self.carried_item);
    }
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct SetContainerProperty {
    pub window_id: u8,
    pub property: i16,
    pub value: i16,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct SetContainerSlot<'a> {
    pub window_id: i8,
    pub state_id: VarInt,
    pub slot: i16,
    /// A single Slot: `bool present + VarInt id + i8 count + NBT`.
    pub slot_data: Slot<'a>,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct SetCooldown {
    pub item_id: VarInt,
    pub cooldown_ticks: VarInt,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct OpenHorseScreen {
    pub window_id: u8,
    pub slot_count: VarInt,
    pub entity_id: i32,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct OpenBook {
    pub hand: VarInt,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct OpenScreen<'a> {
    pub window_id: VarInt,
    pub window_type: VarInt,
    pub window_title: &'a str,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct PlaceGhostRecipe<'a> {
    pub window_id: u8,
    pub recipe: &'a str,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct SetHeldItem {
    pub slot: u8,
}

/// One trade offered by a villager.
#[derive(Debug, Clone)]
pub struct TradeOffer<'a> {
    pub input_item_1: Slot<'a>,
    pub output_item: Slot<'a>,
    /// Only present when the trade's `has_second_item` flag is set.
    pub input_item_2: Option<Slot<'a>>,
    pub trade_disabled: bool,
    pub number_of_trade_uses: i32,
    pub maximum_number_of_trade_uses: i32,
    pub xp: i32,
    pub special_price: i32,
    pub price_multiplier: f32,
    pub demand: i32,
}

/// `Merchant Offers` (0x27).
///
/// Hand-written `Decode`: the second input slot is gated by a boolean, and the
/// packet ends with four villager-level fields that belong to the window rather
/// than to a single trade.
#[derive(Debug, Clone)]
pub struct MerchantOffers<'a> {
    pub window_id: VarInt,
    pub trades: Vec<TradeOffer<'a>>,
    /// 1: Novice, 2: Apprentice, 3: Journeyman, 4: Expert, 5: Master.
    pub villager_level: VarInt,
    /// Total experience of the villager; always 0 for the wandering trader.
    pub experience: VarInt,
    /// False for the wandering trader.
    pub is_regular_villager: bool,
    /// True for regular villagers, false for the wandering trader.
    pub can_restock: bool,
}

impl<'a> Decode<'a> for MerchantOffers<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let window_id = decoder.varint()?;
        let trades = counted(decoder, |d| {
            let input_item_1 = Slot::decode(d)?;
            let output_item = Slot::decode(d)?;
            let input_item_2 = if d.bool()? {
                Some(Slot::decode(d)?)
            } else {
                None
            };
            Ok(TradeOffer {
                input_item_1,
                output_item,
                input_item_2,
                trade_disabled: d.bool()?,
                number_of_trade_uses: d.i32()?,
                maximum_number_of_trade_uses: d.i32()?,
                xp: d.i32()?,
                special_price: d.i32()?,
                price_multiplier: d.f32()?,
                demand: d.i32()?,
            })
        })?;
        let villager_level = decoder.varint()?;
        let experience = decoder.varint()?;
        let is_regular_villager = decoder.bool()?;
        let can_restock = decoder.bool()?;
        Ok(Self {
            window_id,
            trades,
            villager_level,
            experience,
            is_regular_villager,
            can_restock,
        })
    }
}

impl Encode for TradeOffer<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.slot(&self.input_item_1);
        encoder.slot(&self.output_item);
        match &self.input_item_2 {
            Some(slot) => {
                encoder.bool(true);
                encoder.slot(slot);
            }
            None => encoder.bool(false),
        }
        encoder.bool(self.trade_disabled);
        encoder.i32(self.number_of_trade_uses);
        encoder.i32(self.maximum_number_of_trade_uses);
        encoder.i32(self.xp);
        encoder.i32(self.special_price);
        encoder.f32(self.price_multiplier);
        encoder.i32(self.demand);
    }
}

impl Encode for MerchantOffers<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.varint(self.window_id.get());
        encoder.varint(self.trades.len() as i32);
        for trade in &self.trades {
            trade.encode(encoder);
        }
        encoder.varint(self.villager_level.get());
        encoder.varint(self.experience.get());
        encoder.bool(self.is_regular_villager);
        encoder.bool(self.can_restock);
    }
}

#[derive(Debug, Clone)]
pub struct UpdateRecipeBook<'a> {
    pub action: VarInt,
    pub crafting_book_open: bool,
    pub crafting_filter_active: bool,
    pub smelting_book_open: bool,
    pub smelting_filter_active: bool,
    pub blast_furnace_open: bool,
    pub blast_furnace_filter_active: bool,
    pub smoker_open: bool,
    pub smoker_filter_active: bool,
    /// Recipes being added or removed.
    pub recipe_ids: Vec<&'a str>,
    /// Only present for `action == 0` (init), where it lists what is already
    /// known.
    pub already_known: Option<Vec<&'a str>>,
}

impl<'a> Decode<'a> for UpdateRecipeBook<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let action = decoder.varint()?;
        let crafting_book_open = decoder.bool()?;
        let crafting_filter_active = decoder.bool()?;
        let smelting_book_open = decoder.bool()?;
        let smelting_filter_active = decoder.bool()?;
        let blast_furnace_open = decoder.bool()?;
        let blast_furnace_filter_active = decoder.bool()?;
        let smoker_open = decoder.bool()?;
        let smoker_filter_active = decoder.bool()?;
        let recipe_ids = identifiers(decoder)?;
        // Only the "init" action carries the second array.
        let already_known = if action.get() == 0 {
            Some(identifiers(decoder)?)
        } else {
            None
        };
        Ok(Self {
            action,
            crafting_book_open,
            crafting_filter_active,
            smelting_book_open,
            smelting_filter_active,
            blast_furnace_open,
            blast_furnace_filter_active,
            smoker_open,
            smoker_filter_active,
            recipe_ids,
            already_known,
        })
    }
}

impl Encode for UpdateRecipeBook<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.varint(self.action.get());
        encoder.bool(self.crafting_book_open);
        encoder.bool(self.crafting_filter_active);
        encoder.bool(self.smelting_book_open);
        encoder.bool(self.smelting_filter_active);
        encoder.bool(self.blast_furnace_open);
        encoder.bool(self.blast_furnace_filter_active);
        encoder.bool(self.smoker_open);
        encoder.bool(self.smoker_filter_active);
        encoder.encode(&self.recipe_ids);
        // Only the "init" action carries the second array.
        if let Some(already_known) = &self.already_known {
            encoder.encode(already_known);
        }
    }
}
