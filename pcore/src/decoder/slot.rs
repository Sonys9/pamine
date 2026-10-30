//! Inventory slot.
//!
//! Wire format:
//!
//! ```text
//! present: bool
//! if present:
//!     item_id: VarInt
//!     count:   i8
//!     nbt:     NBT (TAG_End when the item has no extra data)
//! ```

use crate::errors::Error;

use super::decoder::Decoder;
use super::nbt::NbtTag;
use super::varint::VarInt;

/// One inventory slot. `None`-like state is `present == false`.
#[derive(Debug, Clone, PartialEq)]
pub struct Slot<'a> {
    pub present: bool,
    pub item_id: Option<VarInt>,
    pub count: Option<i8>,
    pub nbt: Option<NbtTag<'a>>,
}

impl<'a> Slot<'a> {
    /// An empty slot.
    pub fn empty() -> Self {
        Self {
            present: false,
            item_id: None,
            count: None,
            nbt: None,
        }
    }

    /// True when the slot holds nothing.
    pub fn is_empty(&self) -> bool {
        !self.present
    }
}

impl<'a> crate::Decode<'a> for Slot<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let present = decoder.bool()?;
        if !present {
            return Ok(Self::empty());
        }
        Ok(Self {
            present: true,
            item_id: Some(decoder.varint()?),
            count: Some(decoder.i8()?),
            nbt: Some(decoder.nbt()?),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Decode;

    #[test]
    fn empty_slot_is_one_byte() {
        let mut d = Decoder::new(&[0x00]);
        let slot = Slot::decode(&mut d).unwrap();
        assert!(!slot.present);
        assert!(slot.is_empty());
        assert_eq!(d.remaining(), 0);
    }

    #[test]
    fn filled_slot_reads_id_count_and_nbt() {
        // present=1, item_id=5 (VarInt), count=3, nbt=TAG_End
        let mut d = Decoder::new(&[0x01, 0x05, 0x03, 0x00]);
        let slot = Slot::decode(&mut d).unwrap();
        assert!(slot.present);
        assert_eq!(slot.item_id.unwrap().get(), 5);
        assert_eq!(slot.count, Some(3));
        assert_eq!(slot.nbt, Some(NbtTag::End));
        assert_eq!(d.remaining(), 0);
    }

    #[test]
    fn filled_slot_with_compound_nbt() {
        // present=1, item_id=1, count=1, nbt={x:7}
        // Network NBT has no root name, so the compound starts right with its
        // first entry: TAG_Int, name "x", value 7, then TAG_End.
        let data = [
            0x01, 0x01, 0x01, // present, id, count
            0x0A, // compound
            0x03, 0x00, 0x01, b'x', 0x00, 0x00, 0x00, 0x07, // x = 7
            0x00, // end
        ];
        let mut d = Decoder::new(&data);
        let slot = Slot::decode(&mut d).unwrap();
        assert_eq!(slot.item_id.unwrap().get(), 1);
        let nbt = slot.nbt.unwrap();
        assert!(matches!(nbt, NbtTag::Compound(_)));
        assert_eq!(d.remaining(), 0);
    }
}
