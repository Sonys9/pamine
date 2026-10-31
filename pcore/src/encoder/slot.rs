//! Inventory slot writer.
//!
//! Wire format, mirroring [`decoder::slot`](crate::decoder::slot):
//!
//! ```text
//! present: bool
//! if present:
//!     item_id: VarInt
//!     count:   i8
//!     nbt:     NBT (TAG_End when the item has no extra data)
//! ```

use crate::decoder::Slot;

use super::nbt::write_nbt;
use super::varint::write_varint;

/// Appends an inventory slot.
pub fn write_slot(out: &mut Vec<u8>, slot: &Slot<'_>) {
    if !slot.present {
        out.push(0);
        return;
    }
    out.push(1);
    // A present slot always carries an id, a count and an NBT tag; the decoder
    // fills all three together, so the fallbacks are unreachable.
    match &slot.item_id {
        Some(id) => write_varint(out, id.get()),
        None => write_varint(out, 0),
    }
    out.push(slot.count.unwrap_or(0) as u8);
    match &slot.nbt {
        Some(nbt) => write_nbt(out, nbt),
        None => out.push(0),
    }
}

impl crate::Encode for Slot<'_> {
    fn encode(&self, encoder: &mut crate::encoder::Encoder) {
        encoder.slot(self);
    }
}
