//! Block changes, block entities and section updates.

use pcore::{
    Decode,
    decoder::{Decoder, NbtTag, VarInt, VarLong},
    errors::Error,
};

use crate::packets::counted;

#[derive(Decode, Debug, Clone)]
pub struct AcknowledgeBlockChange {
    pub sequence_id: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct SetBlockDestroyStage {
    pub entity_id: VarInt,
    pub location: u64,
    pub destroy_stage: i8,
}

#[derive(Decode, Debug, Clone)]
pub struct BlockEntityData<'a> {
    pub location: u64,
    pub kind: VarInt,
    pub nbt: NbtTag<'a>,
}

#[derive(Decode, Debug, Clone)]
pub struct BlockAction {
    pub location: u64,
    pub action_id: u8,
    pub action_parameter: u8,
    pub block_type: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct BlockUpdate {
    pub location: u64,
    pub block_id: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct OpenSignEditor {
    pub location: u64,
}

/// One changed block inside a section: a packed `position` VarLong and the new
/// block state id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionBlock {
    pub position: VarLong,
    pub block_state: VarInt,
}

#[derive(Debug, Clone)]
pub struct UpdateSectionBlocks {
    pub chunk_section_position: i64,
    pub suppress_light_updates: bool,
    pub blocks: Vec<SectionBlock>,
}

impl<'a> Decode<'a> for UpdateSectionBlocks {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let chunk_section_position = decoder.i64()?;
        let suppress_light_updates = decoder.bool()?;
        let blocks = counted(decoder, |d| {
            Ok(SectionBlock {
                position: d.varlong()?,
                block_state: d.varint()?,
            })
        })?;
        Ok(Self {
            chunk_section_position,
            suppress_light_updates,
            blocks,
        })
    }
}
