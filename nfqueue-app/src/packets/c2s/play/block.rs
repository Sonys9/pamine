//! Block interaction and block-entity editing.

use pcore::{
    Decode,
    decoder::{VarInt, VarLong},
};

#[derive(Decode, Debug, Clone)]
pub struct QueryBlockEntityTag {
    pub transaction_id: VarInt,
    pub location: u64,
}

#[derive(Decode, Debug, Clone)]
pub struct JigsawGenerate {
    pub location: u64,
    pub levels: VarInt,
    pub keep_jigsaws: bool,
}

#[derive(Decode, Debug, Clone)]
pub struct PlayerAction {
    pub status: VarInt,
    pub location: u64,
    pub face: u8,
    pub sequence: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct UseItemOn {
    pub hand: VarInt,
    pub location: u64,
    pub face: VarInt,
    pub cursor_x: f32,
    pub cursor_y: f32,
    pub cursor_z: f32,
    pub inside_block: bool,
    pub sequence: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct UseItem {
    pub hand: VarInt,
    pub sequence: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct UpdateSign<'a> {
    pub location: u64,
    pub line_1: &'a str,
    pub line_2: &'a str,
    pub line_3: &'a str,
    pub line_4: &'a str,
}

#[derive(Decode, Debug, Clone)]
pub struct ProgramCommandBlock<'a> {
    pub location: u64,
    pub command: &'a str,
    pub mode: VarInt,
    pub flags: u8,
}

#[derive(Decode, Debug, Clone)]
pub struct ProgramCommandBlockMinecart<'a> {
    pub entity_id: VarInt,
    pub command: &'a str,
    pub track_output: bool,
}

#[derive(Decode, Debug, Clone)]
pub struct ProgramJigsawBlock<'a> {
    pub location: u64,
    pub name: &'a str,
    pub target: &'a str,
    pub pool: &'a str,
    pub final_state: &'a str,
    pub joint_type: &'a str,
}

#[derive(Decode, Debug, Clone)]
pub struct ProgramStructureBlock<'a> {
    pub location: u64,
    pub action: VarInt,
    pub mode: VarInt,
    pub name: &'a str,
    pub offset_x: i8,
    pub offset_y: i8,
    pub offset_z: i8,
    pub size_x: i8,
    pub size_y: i8,
    pub size_z: i8,
    pub mirror: VarInt,
    pub rotation: VarInt,
    pub metadata: &'a str,
    pub integrity: f32,
    /// VarLong on the wire, not a fixed-width `i64`.
    pub seed: VarLong,
    pub flags: u8,
}
