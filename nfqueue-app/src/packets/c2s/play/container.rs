//! Container clicks and creative-mode slot edits.

use pcore::{
    Decode,
    decoder::{Decoder, Slot, VarInt},
    errors::Error,
};

#[derive(Decode, Debug, Clone)]
pub struct ClickContainerButton {
    pub window_id: i8,
    pub button_id: i8,
}

#[derive(Debug, Clone)]
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

#[derive(Decode, Debug, Clone)]
pub struct CloseContainer {
    pub window_id: u8,
}

#[derive(Decode, Debug, Clone)]
pub struct SetCreativeModeSlot<'a> {
    pub slot: i16,
    /// A single Slot: `bool present + VarInt id + i8 count + NBT`.
    pub clicked_item: Slot<'a>,
}
