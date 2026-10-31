//! Particle payloads (protocol 760 / 1.19.2).
//!
//! A particle is a `VarInt` type id followed by a type-dependent payload. The
//! payload is needed in two places: the `Particle` packet (0x23) and the
//! `Particle` metadata value (type 15), where it sits in the middle of the
//! packet and therefore has to be consumed exactly.
//!
//! The id table below is the 1.19.2 `minecraft:particle_type` registry order.
//! Only the types that carry a payload are listed; every other id has no data.
//! Note that `dust` and `dust_color_transition` use three floats per colour in
//! 1.19.2 — the packed `Int` colour only arrived in 1.20.5.

use pcore::{
    Decode,
    decoder::{Decoder, Slot, VarInt},
    encoder::Encoder,
    errors::Error,
};

/// The payload of a particle, keyed by its type id.
#[derive(Debug, Clone)]
pub enum ParticleData<'a> {
    /// The particle carries no payload.
    None,
    /// `block`, `block_marker`, `falling_dust`.
    BlockState(VarInt),
    /// `dust`: colour components and scale.
    Dust {
        red: f32,
        green: f32,
        blue: f32,
        scale: f32,
    },
    /// `dust_color_transition`: start colour, scale, end colour.
    DustColorTransition {
        from: [f32; 3],
        scale: f32,
        to: [f32; 3],
    },
    /// `item`: the item stack that is drawn.
    Item(Slot<'a>),
    /// `vibration`: where the vibration came from and how long it travels.
    Vibration {
        source: VibrationSource,
        ticks: VarInt,
    },
    /// `sculk_charge`: how much the particle is rotated.
    SculkCharge { roll: f32 },
    /// `shriek`: delay before the particle appears.
    Shriek { delay: VarInt },
}

/// The origin of a `vibration` particle.
#[derive(Debug, Clone, Copy)]
pub enum VibrationSource {
    /// `0` — a block position.
    Block(u64),
    /// `1` — an entity and its eye height.
    Entity { entity_id: VarInt, eye_height: f32 },
}

/// 1.19.2 particle ids that carry a payload.
const PARTICLE_BLOCK: i32 = 3;
const PARTICLE_BLOCK_MARKER: i32 = 4;
const PARTICLE_DUST: i32 = 24;
const PARTICLE_DUST_COLOR_TRANSITION: i32 = 25;
const PARTICLE_FALLING_DUST: i32 = 37;
const PARTICLE_ITEM: i32 = 53;
const PARTICLE_SCULK_CHARGE: i32 = 69;
const PARTICLE_SHRIEK: i32 = 72;
const PARTICLE_VIBRATION: i32 = 86;

/// Reads the payload that follows a particle type id.
///
/// An unknown id is an error rather than a silent no-op: guessing would leave
/// the cursor in the wrong place and corrupt every field after it.
pub fn particle_data<'a>(
    decoder: &mut Decoder<'a>,
    particle_id: i32,
) -> Result<ParticleData<'a>, Error> {
    Ok(match particle_id {
        PARTICLE_BLOCK | PARTICLE_BLOCK_MARKER | PARTICLE_FALLING_DUST => {
            ParticleData::BlockState(decoder.varint()?)
        }
        PARTICLE_DUST => ParticleData::Dust {
            red: decoder.f32()?,
            green: decoder.f32()?,
            blue: decoder.f32()?,
            scale: decoder.f32()?,
        },
        PARTICLE_DUST_COLOR_TRANSITION => ParticleData::DustColorTransition {
            from: [decoder.f32()?, decoder.f32()?, decoder.f32()?],
            scale: decoder.f32()?,
            to: [decoder.f32()?, decoder.f32()?, decoder.f32()?],
        },
        PARTICLE_ITEM => ParticleData::Item(Slot::decode(decoder)?),
        PARTICLE_VIBRATION => {
            let source = match decoder.varint()?.get() {
                0 => VibrationSource::Block(decoder.u64()?),
                1 => VibrationSource::Entity {
                    entity_id: decoder.varint()?,
                    eye_height: decoder.f32()?,
                },
                other => return Err(Error::UnknownOpcode(other)),
            };
            ParticleData::Vibration {
                source,
                ticks: decoder.varint()?,
            }
        }
        PARTICLE_SCULK_CHARGE => ParticleData::SculkCharge {
            roll: decoder.f32()?,
        },
        PARTICLE_SHRIEK => ParticleData::Shriek {
            delay: decoder.varint()?,
        },
        // Every other 1.19.2 particle is payload-free.
        _ => ParticleData::None,
    })
}

/// Writes the payload of a particle, without the type id.
///
/// The variant itself says which layout to use, so the id is not needed here.
pub fn write_particle_data(encoder: &mut Encoder, data: &ParticleData<'_>) {
    match data {
        ParticleData::None => {}
        ParticleData::BlockState(state) => encoder.varint(state.get()),
        ParticleData::Dust {
            red,
            green,
            blue,
            scale,
        } => {
            encoder.f32(*red);
            encoder.f32(*green);
            encoder.f32(*blue);
            encoder.f32(*scale);
        }
        ParticleData::DustColorTransition { from, scale, to } => {
            for channel in from {
                encoder.f32(*channel);
            }
            encoder.f32(*scale);
            for channel in to {
                encoder.f32(*channel);
            }
        }
        ParticleData::Item(slot) => encoder.slot(slot),
        ParticleData::Vibration { source, ticks } => {
            match source {
                VibrationSource::Block(position) => {
                    encoder.varint(0);
                    encoder.u64(*position);
                }
                VibrationSource::Entity {
                    entity_id,
                    eye_height,
                } => {
                    encoder.varint(1);
                    encoder.varint(entity_id.get());
                    encoder.f32(*eye_height);
                }
            }
            encoder.varint(ticks.get());
        }
        ParticleData::SculkCharge { roll } => encoder.f32(*roll),
        ParticleData::Shriek { delay } => encoder.varint(delay.get()),
    }
}
