//! Sound effects.

use pcore::{
    Decode,
    decoder::{Decoder, VarInt},
    errors::Error,
};

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

/// `Entity Sound Effect` (0x5F). Unlike [`SoundEffect`] it carries no seed.
#[derive(Decode, Debug, Clone)]
pub struct EntitySoundEffect {
    pub sound_id: VarInt,
    pub sound_category: VarInt,
    pub entity_id: VarInt,
    pub volume: f32,
    pub pitch: f32,
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

/// `Stop Sound` (0x61).
///
/// Hand-written `Decode`: `source` is gated by bit 0x1 of `flags` and `sound`
/// by bit 0x2, not by per-field booleans.
#[derive(Debug, Clone)]
pub struct StopSound<'a> {
    pub flags: u8,
    pub source: Option<VarInt>,
    pub sound: Option<&'a str>,
}

impl<'a> Decode<'a> for StopSound<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        const SOURCE: u8 = 0x01;
        const SOUND: u8 = 0x02;

        let flags = decoder.u8()?;
        let source = if flags & SOURCE != 0 {
            Some(decoder.varint()?)
        } else {
            None
        };
        let sound = if flags & SOUND != 0 {
            Some(decoder.string()?)
        } else {
            None
        };
        Ok(Self {
            flags,
            source,
            sound,
        })
    }
}
