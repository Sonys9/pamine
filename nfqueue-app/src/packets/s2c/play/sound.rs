//! Sound effects.

use pcore::{Decode, decoder::VarInt};

#[derive(Decode, Debug, Clone)]
pub struct CustomSoundEffect<'a> {
    pub sound_name: &'a str,
    pub sound_category: VarInt,
    pub effect_position_x: i32,
    pub effect_position_y: i32,
    pub effect_position_z: i32,
    pub volume: f32,
    pub pitch: f32,
    pub seed: i64,
}

#[derive(Decode, Debug, Clone)]
pub struct EntitySoundEffect {
    pub sound_id: VarInt,
    pub sound_category: VarInt,
    pub entity_id: VarInt,
    pub volume: f32,
    pub pitch: f32,
    pub seed: i64,
}

#[derive(Decode, Debug, Clone)]
pub struct SoundEffect {
    pub sound_id: VarInt,
    pub sound_category: VarInt,
    pub effect_position_x: i32,
    pub effect_position_y: i32,
    pub effect_position_z: i32,
    pub volume: f32,
    pub pitch: f32,
    pub seed: i64,
}

#[derive(Decode, Debug, Clone)]
pub struct StopSound<'a> {
    pub flags: u8,
    pub source: Option<VarInt>,
    pub sound: Option<&'a str>,
}
