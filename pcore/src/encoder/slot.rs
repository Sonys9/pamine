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

use super::varint::write_varint;

impl crate::Encode for Slot<'_> {
    fn encode(&self, out: &mut Vec<u8>) {
        if !self.present {
            out.push(0);
            return;
        }
        out.push(1);
        // A present slot always carries an id, a count and an NBT tag; the
        // decoder fills all three together, so the fallbacks are unreachable.
        match &self.item_id {
            Some(id) => id.encode(out),
            None => write_varint(out, 0),
        }
        out.push(self.count.unwrap_or(0) as u8);
        match &self.nbt {
            Some(nbt) => nbt.encode(out),
            None => out.push(0),
        }
    }
}
