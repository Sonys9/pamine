//! World state: border, time, events, explosions, particles and maps.

use pcore::{
    Decode,
    decoder::{Decoder, VarInt},
    errors::Error,
};

use crate::packets::{counted, optional_chat};

#[derive(Decode, Debug, Clone)]
pub struct InitializeWorldBorder {
    pub x: f64,
    pub z: f64,
    pub old_diameter: f64,
    pub new_diameter: f64,
    pub speed: i64,
    pub portal_teleport_boundary: VarInt,
    pub warning_blocks: VarInt,
    pub warning_time: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct SetBorderCenter {
    pub x: f64,
    pub z: f64,
}

#[derive(Decode, Debug, Clone)]
pub struct SetBorderLerpSize {
    pub old_diameter: f64,
    pub new_diameter: f64,
    pub speed: i64,
}

#[derive(Decode, Debug, Clone)]
pub struct SetBorderSize {
    pub diameter: f64,
}

#[derive(Decode, Debug, Clone)]
pub struct SetBorderWarningDelay {
    pub warning_time: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct SetBorderWarningDistance {
    pub warning_blocks: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct GameEvent {
    pub event: u8,
    pub value: f32,
}

#[derive(Decode, Debug, Clone)]
pub struct WorldEvent {
    pub event: i32,
    pub location: u64,
    pub data: i32,
    pub disable_relative_volume: bool,
}

#[derive(Decode, Debug, Clone)]
pub struct UpdateTime {
    pub world_age: i64,
    pub time_of_day: i64,
}

/// One destroyed block, as a signed offset from the explosion centre.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExplosionRecord {
    pub x: i8,
    pub y: i8,
    pub z: i8,
}

#[derive(Debug, Clone)]
pub struct Explosion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub strength: f32,
    pub records: Vec<ExplosionRecord>,
    /// The knockback applied to the player, one float per axis.
    pub player_motion: [f32; 3],
}

impl<'a> Decode<'a> for Explosion {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let x = decoder.f32()?;
        let y = decoder.f32()?;
        let z = decoder.f32()?;
        let strength = decoder.f32()?;
        let records = counted(decoder, |d| {
            Ok(ExplosionRecord {
                x: d.i8()?,
                y: d.i8()?,
                z: d.i8()?,
            })
        })?;
        let player_motion = [decoder.f32()?, decoder.f32()?, decoder.f32()?];
        Ok(Self {
            x,
            y,
            z,
            strength,
            records,
            player_motion,
        })
    }
}

#[derive(Decode, Debug, Clone)]
pub struct Particle<'a> {
    pub particle_id: VarInt,
    pub long_distance: bool,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub offset_x: f32,
    pub offset_y: f32,
    pub offset_z: f32,
    pub max_speed: f32,
    pub particle_count: i32,
    pub data: &'a [u8],
}

/// One marker drawn on a map.
#[derive(Debug, Clone)]
pub struct MapIcon<'a> {
    /// `type << 4 | direction`, packed into a single byte.
    pub kind_and_direction: u8,
    pub x: i8,
    pub z: i8,
    /// A `bool`-gated display name.
    pub display_name: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct MapData<'a> {
    pub map_id: VarInt,
    pub scale: i8,
    pub locked: bool,
    pub icons: Vec<MapIcon<'a>>,
}

impl<'a> Decode<'a> for MapData<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let map_id = decoder.varint()?;
        let scale = decoder.i8()?;
        let locked = decoder.bool()?;
        // The icon array is only on the wire when tracking is enabled; the
        // count is a VarInt and `0` covers the "not tracking" case.
        let icons = counted(decoder, |d| {
            Ok(MapIcon {
                kind_and_direction: d.u8()?,
                x: d.i8()?,
                z: d.i8()?,
                display_name: optional_chat(d)?,
            })
        })?;
        Ok(Self {
            map_id,
            scale,
            locked,
            icons,
        })
    }
}
