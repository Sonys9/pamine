//! World state: border, time, events, explosions, particles and maps.

use pcore::{
    Decode, Encode,
    decoder::{Decoder, VarInt, VarLong},
    encoder::Encoder,
    errors::Error,
};

use super::particle::{ParticleData, particle_data, write_particle_data};
use crate::packets::{counted, optional_chat};

#[derive(Decode, Encode, Debug, Clone)]
pub struct InitializeWorldBorder {
    pub x: f64,
    pub z: f64,
    pub old_diameter: f64,
    pub new_diameter: f64,
    /// Milliseconds until `new_diameter` is reached; `0` when not moving.
    /// VarLong on the wire, not a fixed-width `i64`.
    pub speed: VarLong,
    pub portal_teleport_boundary: VarInt,
    pub warning_blocks: VarInt,
    pub warning_time: VarInt,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct SetBorderCenter {
    pub x: f64,
    pub z: f64,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct SetBorderLerpSize {
    pub old_diameter: f64,
    pub new_diameter: f64,
    /// Milliseconds until `new_diameter` is reached; `0` when not moving.
    pub speed: VarLong,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct SetBorderSize {
    pub diameter: f64,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct SetBorderWarningDelay {
    pub warning_time: VarInt,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct SetBorderWarningDistance {
    pub warning_blocks: VarInt,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct GameEvent {
    pub event: u8,
    pub value: f32,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct WorldEvent {
    pub event: i32,
    pub location: u64,
    pub data: i32,
    pub disable_relative_volume: bool,
}

#[derive(Decode, Encode, Debug, Clone)]
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

impl Encode for Explosion {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.f32(self.x);
        encoder.f32(self.y);
        encoder.f32(self.z);
        encoder.f32(self.strength);
        encoder.varint(self.records.len() as i32);
        for record in &self.records {
            encoder.i8(record.x);
            encoder.i8(record.y);
            encoder.i8(record.z);
        }
        for axis in self.player_motion {
            encoder.f32(axis);
        }
    }
}

/// `Particle` (0x23).
///
/// Hand-written `Decode`: the trailing `Data` field is a union keyed by
/// `particle_id`, so it is read through [`particle_data`] rather than left as
/// raw bytes.
#[derive(Debug, Clone)]
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
    pub data: ParticleData<'a>,
}

impl<'a> Decode<'a> for Particle<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let particle_id = decoder.varint()?;
        let long_distance = decoder.bool()?;
        let x = decoder.f64()?;
        let y = decoder.f64()?;
        let z = decoder.f64()?;
        let offset_x = decoder.f32()?;
        let offset_y = decoder.f32()?;
        let offset_z = decoder.f32()?;
        let max_speed = decoder.f32()?;
        let particle_count = decoder.i32()?;
        let data = particle_data(decoder, particle_id.get())?;
        Ok(Self {
            particle_id,
            long_distance,
            x,
            y,
            z,
            offset_x,
            offset_y,
            offset_z,
            max_speed,
            particle_count,
            data,
        })
    }
}

impl Encode for Particle<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.varint(self.particle_id.get());
        encoder.bool(self.long_distance);
        encoder.f64(self.x);
        encoder.f64(self.y);
        encoder.f64(self.z);
        encoder.f32(self.offset_x);
        encoder.f32(self.offset_y);
        encoder.f32(self.offset_z);
        encoder.f32(self.max_speed);
        encoder.i32(self.particle_count);
        write_particle_data(encoder, &self.data);
    }
}

/// One marker drawn on a map.
#[derive(Debug, Clone)]
pub struct MapIcon<'a> {
    /// Icon type, see the map icon table.
    pub kind: VarInt,
    /// Map coordinate: -128 is furthest left, +127 furthest right.
    pub x: i8,
    /// Map coordinate: -128 is highest, +127 lowest.
    pub z: i8,
    /// 0-15; 0 is a vertical icon and each step is 22.5°.
    pub direction: i8,
    /// A `bool`-gated display name.
    pub display_name: Option<&'a str>,
}

/// `Map Data` (0x26).
///
/// Hand-written `Decode`: the icon array is gated by `tracking_position`, and
/// the pixel block is gated by `columns > 0`.
#[derive(Debug, Clone)]
pub struct MapData<'a> {
    pub map_id: VarInt,
    pub scale: i8,
    pub locked: bool,
    pub tracking_position: bool,
    /// Only present when `tracking_position` is set.
    pub icons: Vec<MapIcon<'a>>,
    pub columns: u8,
    /// Only present when `columns > 0`.
    pub rows: Option<u8>,
    /// Only present when `columns > 0`.
    pub x: Option<i8>,
    /// Only present when `columns > 0`.
    pub z: Option<i8>,
    /// Only present when `columns > 0`.
    pub data: Option<&'a [u8]>,
}

impl<'a> Decode<'a> for MapData<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let map_id = decoder.varint()?;
        let scale = decoder.i8()?;
        let locked = decoder.bool()?;
        let tracking_position = decoder.bool()?;

        let icons = if tracking_position {
            counted(decoder, |d| {
                Ok(MapIcon {
                    kind: d.varint()?,
                    x: d.i8()?,
                    z: d.i8()?,
                    direction: d.i8()?,
                    display_name: optional_chat(d)?,
                })
            })?
        } else {
            Vec::new()
        };

        let columns = decoder.u8()?;
        let (rows, x, z, data) = if columns > 0 {
            let rows = decoder.u8()?;
            let x = decoder.i8()?;
            let z = decoder.i8()?;
            // The pixel array is length-prefixed by a VarInt.
            let data = decoder.slice()?;
            (Some(rows), Some(x), Some(z), Some(data))
        } else {
            (None, None, None, None)
        };

        Ok(Self {
            map_id,
            scale,
            locked,
            tracking_position,
            icons,
            columns,
            rows,
            x,
            z,
            data,
        })
    }
}

impl Encode for MapIcon<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.varint(self.kind.get());
        encoder.i8(self.x);
        encoder.i8(self.z);
        encoder.i8(self.direction);
        match self.display_name {
            Some(name) => {
                encoder.bool(true);
                encoder.string(name);
            }
            None => encoder.bool(false),
        }
    }
}

impl Encode for MapData<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.varint(self.map_id.get());
        encoder.i8(self.scale);
        encoder.bool(self.locked);
        encoder.bool(self.tracking_position);
        if self.tracking_position {
            encoder.varint(self.icons.len() as i32);
            for icon in &self.icons {
                icon.encode(encoder);
            }
        }
        encoder.u8(self.columns);
        if self.columns > 0 {
            encoder.u8(self.rows.unwrap_or(0));
            encoder.i8(self.x.unwrap_or(0));
            encoder.i8(self.z.unwrap_or(0));
            encoder.bytes(self.data.unwrap_or(&[]));
        }
    }
}
