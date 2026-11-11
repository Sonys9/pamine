//! Container clicks and creative-mode slot edits.

use pcore::{
    Decode, Encode,
    decoder::{Decoder, Slot, VarInt},
    encoder::Encoder,
    errors::Error,
};

use pcore_gen::{Lua, LuaPacket};

#[derive(Decode, Encode, Lua, LuaPacket, Debug, Clone)]
pub struct ClickContainerButton {
    pub window_id: i8,
    pub button_id: i8,
}

#[derive(Lua, Debug, Clone)]
pub struct ClickContainer<'a> {
    pub window_id: u8,
    pub state_id: VarInt,
    pub slot: i16,
    pub button: i8,
    pub mode: VarInt,
    /// `(slot number, Slot)` pairs the client changed.
    pub changed_slots: Vec<(i16, Slot<'a>)>,
    /// The item still held by the cursor after the click.
    pub carried_item: Slot<'a>,
}

/// Hand-written: `changed_slots` is a `Vec<(i16, Slot)>`, and a tuple has no
/// `lua_encode_table` of its own. Each entry is read as a two-element Lua
/// sequence `{ slot_number, slot }`.
impl ClickContainer<'_> {
    pub fn lua_encode_table(table: &mlua::Table, encoder: &mut Encoder) -> mlua::Result<()> {
        encoder.u8(table.get::<u8>("window_id")?);
        encoder.varint(table.get::<i32>("state_id")?);
        encoder.i16(table.get::<i16>("slot")?);
        encoder.i8(table.get::<i8>("button")?);
        encoder.varint(table.get::<i32>("mode")?);

        let changed: Vec<mlua::Table> = table.get("changed_slots")?;
        encoder.varint(changed.len() as i32);
        for entry in changed {
            encoder.i16(entry.get::<i16>(1)?);
            let slot: pcore::lua::OwnedSlot = entry.get(2)?;
            encoder.slot(&slot.as_borrowed());
        }

        let carried: pcore::lua::OwnedSlot = table.get("carried_item")?;
        encoder.slot(&carried.as_borrowed());
        Ok(())
    }
}

impl<'a> Decode<'a> for ClickContainer<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let window_id = decoder.u8()?;
        let state_id = decoder.varint()?;
        let slot = decoder.i16()?;
        let button = decoder.i8()?;
        let mode = decoder.varint()?;

        // The count is a VarInt, and each entry is a bare i16 followed by a Slot.
        let count = decoder.varint()?.get();
        if count < 0 {
            return Err(Error::Read(0));
        }
        let mut changed_slots = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let index = decoder.i16()?;
            changed_slots.push((index, Slot::decode(decoder)?));
        }
        let carried_item = Slot::decode(decoder)?;

        Ok(Self {
            window_id,
            state_id,
            slot,
            button,
            mode,
            changed_slots,
            carried_item,
        })
    }
}

impl Encode for ClickContainer<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.u8(self.window_id);
        encoder.varint(self.state_id.get());
        encoder.i16(self.slot);
        encoder.i8(self.button);
        encoder.varint(self.mode.get());
        // Each entry is a bare i16 followed by a Slot.
        encoder.varint(self.changed_slots.len() as i32);
        for (index, slot) in &self.changed_slots {
            encoder.i16(*index);
            encoder.slot(slot);
        }
        encoder.slot(&self.carried_item);
    }
}

#[derive(Decode, Encode, Lua, LuaPacket, Debug, Clone)]
pub struct CloseContainer {
    pub window_id: u8,
}

#[derive(Decode, Encode, Lua, LuaPacket, Debug, Clone)]
pub struct SetCreativeModeSlot<'a> {
    pub slot: i16,
    /// A single Slot: `bool present + VarInt id + i8 count + NBT`.
    pub clicked_item: Slot<'a>,
}
