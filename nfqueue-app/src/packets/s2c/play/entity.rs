//! Entity lifecycle, movement, metadata and attributes.

use pcore::{
    Decode, Encode,
    decoder::{Decoder, NbtTag, Slot, Uuid, VarInt},
    encoder::Encoder,
    errors::Error,
};

use super::particle::{ParticleData, particle_data, write_particle_data};
use crate::packets::{counted, optional_chat};

#[derive(Decode, Encode, Debug, Clone)]
pub struct SpawnEntity {
    pub entity_id: VarInt,
    pub entity_uuid: Uuid,
    pub kind: VarInt,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub pitch: u8,
    pub yaw: u8,
    pub head_yaw: u8,
    pub data: VarInt,
    pub velocity_x: i16,
    pub velocity_y: i16,
    pub velocity_z: i16,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct SpawnExperienceOrb {
    pub entity_id: VarInt,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub count: i16,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct SpawnPlayer {
    pub entity_id: VarInt,
    pub player_uuid: Uuid,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: u8,
    pub pitch: u8,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct EntityAnimation {
    pub entity_id: VarInt,
    pub animation: u8,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct EntityEvent {
    pub entity_id: i32,
    pub entity_status: i8,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct UpdateEntityPosition {
    pub entity_id: VarInt,
    pub delta_x: i16,
    pub delta_y: i16,
    pub delta_z: i16,
    pub on_ground: bool,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct UpdateEntityPositionAndRotation {
    pub entity_id: VarInt,
    pub delta_x: i16,
    pub delta_y: i16,
    pub delta_z: i16,
    pub yaw: u8,
    pub pitch: u8,
    pub on_ground: bool,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct UpdateEntityRotation {
    pub entity_id: VarInt,
    pub yaw: u8,
    pub pitch: u8,
    pub on_ground: bool,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct MoveVehicle {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
}

/// `Look At` (0x38).
///
/// Hand-written `Decode`: `entity_id` and `entity_feet_eyes` are gated by
/// `is_entity`, not by their own booleans.
#[derive(Debug, Clone)]
pub struct LookAt {
    /// 0 = feet, 1 = eyes.
    pub feet_eyes: VarInt,
    pub target_x: f64,
    pub target_y: f64,
    pub target_z: f64,
    pub is_entity: bool,
    pub entity_id: Option<VarInt>,
    /// 0 = feet, 1 = eyes; same meaning as `feet_eyes`, for the target entity.
    pub entity_feet_eyes: Option<VarInt>,
}

impl<'a> Decode<'a> for LookAt {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let feet_eyes = decoder.varint()?;
        let target_x = decoder.f64()?;
        let target_y = decoder.f64()?;
        let target_z = decoder.f64()?;
        let is_entity = decoder.bool()?;
        let (entity_id, entity_feet_eyes) = if is_entity {
            (Some(decoder.varint()?), Some(decoder.varint()?))
        } else {
            (None, None)
        };
        Ok(Self {
            feet_eyes,
            target_x,
            target_y,
            target_z,
            is_entity,
            entity_id,
            entity_feet_eyes,
        })
    }
}

impl Encode for LookAt {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.varint(self.feet_eyes.get());
        encoder.f64(self.target_x);
        encoder.f64(self.target_y);
        encoder.f64(self.target_z);
        encoder.bool(self.is_entity);
        if self.is_entity {
            encoder.varint(self.entity_id.map(|v| v.get()).unwrap_or(0));
            encoder.varint(self.entity_feet_eyes.map(|v| v.get()).unwrap_or(0));
        }
    }
}

#[derive(Debug, Clone)]
pub struct RemoveEntities {
    pub entity_ids: Vec<VarInt>,
}

impl<'a> Decode<'a> for RemoveEntities {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let entity_ids = counted(decoder, |d| d.varint())?;
        Ok(Self { entity_ids })
    }
}

impl Encode for RemoveEntities {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.encode(&self.entity_ids);
    }
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct RemoveEntityEffect {
    pub entity_id: VarInt,
    pub effect_id: VarInt,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct SetHeadRotation {
    pub entity_id: VarInt,
    pub head_yaw: u8,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct SetCamera {
    pub camera_id: VarInt,
}

/// One metadata entry.
///
/// The wire format is `index: u8, type: VarInt, value: <type>`. The index only
/// says *which* property this is; the type says how to read the value, so the
/// two must not be confused.
#[derive(Debug, Clone)]
pub struct MetadataEntry<'a> {
    pub index: u8,
    pub value: MetadataValue<'a>,
}

/// The metadata value types of protocol 760.
///
/// This is the **1.19.2** table (ids 0-20). It is not the modern one: 1.20.5
/// inserted `VarLong`, `BlockState`, `OptBlockState` and `Particles`, replaced
/// `OptUuid`/`OptBlockId` with `OptLivingEntityRef` and dropped `NBT`, which
/// shifts every id after `Float`. Using the newer table desynchronises every
/// metadata entry.
#[derive(Debug, Clone)]
pub enum MetadataValue<'a> {
    Byte(i8),
    VarInt(VarInt),
    Float(f32),
    String(&'a str),
    Chat(&'a str),
    OptChat(Option<&'a str>),
    Slot(Slot<'a>),
    Boolean(bool),
    Rotation([f32; 3]),
    Position(u64),
    OptPosition(Option<u64>),
    Direction(VarInt),
    OptUuid(Option<Uuid>),
    /// `0` means absent.
    OptBlockId(Option<VarInt>),
    Nbt(NbtTag<'a>),
    Particle {
        particle_id: VarInt,
        data: ParticleData<'a>,
    },
    VillagerData {
        kind: VarInt,
        profession: VarInt,
        level: VarInt,
    },
    /// `0` means absent, otherwise the value is `1 + actual`.
    OptVarInt(Option<VarInt>),
    Pose(VarInt),
    /// An id in the `minecraft:cat_variant` registry.
    CatVariant(VarInt),
    /// An id in the `minecraft:frog_variant` registry.
    FrogVariant(VarInt),
}

impl Encode for MetadataValue<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        match self {
            Self::Byte(v) => {
                encoder.varint(0);
                encoder.i8(*v);
            }
            Self::VarInt(v) => {
                encoder.varint(1);
                encoder.varint(v.get());
            }
            Self::Float(v) => {
                encoder.varint(2);
                encoder.f32(*v);
            }
            Self::String(v) => {
                encoder.varint(3);
                encoder.string(v);
            }
            Self::Chat(v) => {
                encoder.varint(4);
                encoder.string(v);
            }
            Self::OptChat(v) => {
                encoder.varint(5);
                match v {
                    Some(text) => {
                        encoder.bool(true);
                        encoder.string(text);
                    }
                    None => encoder.bool(false),
                }
            }
            Self::Slot(v) => {
                encoder.varint(6);
                encoder.slot(v);
            }
            Self::Boolean(v) => {
                encoder.varint(7);
                encoder.bool(*v);
            }
            Self::Rotation(v) => {
                encoder.varint(8);
                for axis in v {
                    encoder.f32(*axis);
                }
            }
            Self::Position(v) => {
                encoder.varint(9);
                encoder.u64(*v);
            }
            Self::OptPosition(v) => {
                encoder.varint(10);
                match v {
                    Some(position) => {
                        encoder.bool(true);
                        encoder.u64(*position);
                    }
                    None => encoder.bool(false),
                }
            }
            Self::Direction(v) => {
                encoder.varint(11);
                encoder.varint(v.get());
            }
            Self::OptUuid(v) => {
                encoder.varint(12);
                match v {
                    Some(uuid) => {
                        encoder.bool(true);
                        encoder.uuid(uuid);
                    }
                    None => encoder.bool(false),
                }
            }
            Self::OptBlockId(v) => {
                encoder.varint(13);
                encoder.varint(v.map(|id| id.get()).unwrap_or(0));
            }
            Self::Nbt(v) => {
                encoder.varint(14);
                encoder.nbt(v);
            }
            Self::Particle { particle_id, data } => {
                encoder.varint(15);
                encoder.varint(particle_id.get());
                write_particle_data(encoder, data);
            }
            Self::VillagerData {
                kind,
                profession,
                level,
            } => {
                encoder.varint(16);
                encoder.varint(kind.get());
                encoder.varint(profession.get());
                encoder.varint(level.get());
            }
            Self::OptVarInt(v) => {
                encoder.varint(17);
                encoder.varint(match v {
                    Some(value) => value.get() + 1,
                    None => 0,
                });
            }
            Self::Pose(v) => {
                encoder.varint(18);
                encoder.varint(v.get());
            }
            Self::CatVariant(v) => {
                encoder.varint(19);
                encoder.varint(v.get());
            }
            Self::FrogVariant(v) => {
                encoder.varint(20);
                encoder.varint(v.get());
            }
        }
    }
}

impl<'a> MetadataValue<'a> {
    /// Reads the value whose type id has already been consumed.
    fn read(decoder: &mut Decoder<'a>, kind: i32) -> Result<Self, Error> {
        Ok(match kind {
            0 => Self::Byte(decoder.i8()?),
            1 => Self::VarInt(decoder.varint()?),
            2 => Self::Float(decoder.f32()?),
            3 => Self::String(decoder.string()?),
            4 => Self::Chat(decoder.string()?),
            5 => Self::OptChat(optional_chat(decoder)?),
            6 => Self::Slot(Slot::decode(decoder)?),
            7 => Self::Boolean(decoder.bool()?),
            8 => Self::Rotation([decoder.f32()?, decoder.f32()?, decoder.f32()?]),
            9 => Self::Position(decoder.u64()?),
            10 => Self::OptPosition(if decoder.bool()? {
                Some(decoder.u64()?)
            } else {
                None
            }),
            11 => Self::Direction(decoder.varint()?),
            12 => Self::OptUuid(if decoder.bool()? {
                Some(decoder.uuid()?)
            } else {
                None
            }),
            13 => {
                // Read once: the value doubles as the presence marker.
                let id = decoder.varint()?;
                Self::OptBlockId((id.get() != 0).then_some(id))
            }
            14 => Self::Nbt(decoder.nbt()?),
            15 => {
                let particle_id = decoder.varint()?;
                let data = particle_data(decoder, particle_id.get())?;
                Self::Particle { particle_id, data }
            }
            16 => Self::VillagerData {
                kind: decoder.varint()?,
                profession: decoder.varint()?,
                level: decoder.varint()?,
            },
            17 => Self::OptVarInt(match decoder.varint()?.get() {
                0 => None,
                n => Some(VarInt::new(n - 1)),
            }),
            18 => Self::Pose(decoder.varint()?),
            19 => Self::CatVariant(decoder.varint()?),
            20 => Self::FrogVariant(decoder.varint()?),
            other => return Err(Error::UnknownOpcode(other)),
        })
    }
}

/// `Set Entity Metadata` (0x50).
///
/// Hand-written `Decode`: entries are `index, type, value` triples terminated
/// by a `0xFF` index byte, and the value layout comes from the type.
#[derive(Debug, Clone)]
pub struct SetEntityMetadata<'a> {
    pub entity_id: VarInt,
    pub metadata: Vec<MetadataEntry<'a>>,
}

impl<'a> Decode<'a> for SetEntityMetadata<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let entity_id = decoder.varint()?;
        let mut metadata = Vec::new();
        // Entries are terminated by a `0xFF` index byte.
        loop {
            let index = decoder.u8()?;
            if index == 0xFF {
                break;
            }
            let kind = decoder.varint()?.get();
            let value = MetadataValue::read(decoder, kind)?;
            metadata.push(MetadataEntry { index, value });
        }
        Ok(Self {
            entity_id,
            metadata,
        })
    }
}

impl Encode for SetEntityMetadata<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.varint(self.entity_id.get());
        for entry in &self.metadata {
            encoder.u8(entry.index);
            entry.value.encode(encoder);
        }
        encoder.u8(0xFF);
    }
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct LinkEntities {
    pub attached_entity_id: i32,
    pub holding_entity_id: i32,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct SetEntityVelocity {
    pub entity_id: VarInt,
    pub velocity_x: i16,
    pub velocity_y: i16,
    pub velocity_z: i16,
}

/// One equipped item. `slot` is the raw equipment-slot id (0-5).
#[derive(Debug, Clone)]
pub struct EquipmentEntry<'a> {
    pub slot: i8,
    pub item: Slot<'a>,
}

#[derive(Debug, Clone)]
pub struct SetEquipment<'a> {
    pub entity_id: VarInt,
    pub equipment: Vec<EquipmentEntry<'a>>,
}

impl<'a> Decode<'a> for SetEquipment<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        const CONTINUED: u8 = 0x80;
        /// Equipment slots are 0-5; anything else is a malformed packet.
        const MAX_SLOT: u8 = 5;

        let entity_id = decoder.varint()?;
        let mut equipment = Vec::new();
        // The high bit of the slot byte means "another entry follows".
        loop {
            let raw = decoder.u8()?;
            let slot = raw & !CONTINUED;
            if slot > MAX_SLOT {
                return Err(Error::UnknownOpcode(slot as i32));
            }
            let item = Slot::decode(decoder)?;
            equipment.push(EquipmentEntry {
                slot: slot as i8,
                item,
            });
            if raw & CONTINUED == 0 {
                break;
            }
        }
        Ok(Self {
            entity_id,
            equipment,
        })
    }
}

impl Encode for EquipmentEntry<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.i8(self.slot);
        encoder.slot(&self.item);
    }
}

impl Encode for SetEquipment<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        const CONTINUED: u8 = 0x80;

        encoder.varint(self.entity_id.get());
        let last = self.equipment.len().saturating_sub(1);
        for (index, entry) in self.equipment.iter().enumerate() {
            // The high bit marks "another entry follows".
            let raw = entry.slot as u8 | if index < last { CONTINUED } else { 0 };
            encoder.u8(raw);
            encoder.slot(&entry.item);
        }
    }
}

#[derive(Debug, Clone)]
pub struct SetPassengers {
    pub entity_id: VarInt,
    pub passengers: Vec<VarInt>,
}

impl<'a> Decode<'a> for SetPassengers {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let entity_id = decoder.varint()?;
        let passengers = counted(decoder, |d| d.varint())?;
        Ok(Self {
            entity_id,
            passengers,
        })
    }
}

impl Encode for SetPassengers {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.varint(self.entity_id.get());
        encoder.encode(&self.passengers);
    }
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct PickupItem {
    pub collected_entity_id: VarInt,
    pub collector_entity_id: VarInt,
    pub pickup_item_count: VarInt,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct TeleportEntity {
    pub entity_id: VarInt,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: u8,
    pub pitch: u8,
    pub on_ground: bool,
}

/// One attribute modifier.
#[derive(Debug, Clone)]
pub struct AttributeModifier {
    pub uuid: Uuid,
    pub amount: f64,
    pub operation: u8,
}

/// One attribute of an entity, with its base value and modifiers.
#[derive(Debug, Clone)]
pub struct AttributeEntry<'a> {
    pub key: &'a str,
    pub value: f64,
    pub modifiers: Vec<AttributeModifier>,
}

#[derive(Debug, Clone)]
pub struct UpdateAttributes<'a> {
    pub entity_id: VarInt,
    pub properties: Vec<AttributeEntry<'a>>,
}

impl<'a> Decode<'a> for UpdateAttributes<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let entity_id = decoder.varint()?;
        let properties = counted(decoder, |d| {
            let key = d.string()?;
            let value = d.f64()?;
            let modifiers = counted(d, |d| {
                Ok(AttributeModifier {
                    uuid: d.uuid()?,
                    amount: d.f64()?,
                    operation: d.u8()?,
                })
            })?;
            Ok(AttributeEntry {
                key,
                value,
                modifiers,
            })
        })?;
        Ok(Self {
            entity_id,
            properties,
        })
    }
}

impl Encode for AttributeModifier {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.uuid(&self.uuid);
        encoder.f64(self.amount);
        encoder.u8(self.operation);
    }
}

impl Encode for AttributeEntry<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.string(self.key);
        encoder.f64(self.value);
        encoder.varint(self.modifiers.len() as i32);
        for modifier in &self.modifiers {
            modifier.encode(encoder);
        }
    }
}

impl Encode for UpdateAttributes<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.varint(self.entity_id.get());
        encoder.varint(self.properties.len() as i32);
        for property in &self.properties {
            property.encode(encoder);
        }
    }
}

/// `Entity Effect` (0x69).
///
/// `factor_data` is gated by a boolean on the wire; `Option<NbtTag>` reads that
/// boolean itself, so there is no separate `has_factor_data` field.
#[derive(Decode, Encode, Debug, Clone)]
pub struct EntityEffect<'a> {
    pub entity_id: VarInt,
    pub effect_id: VarInt,
    pub amplifier: i8,
    pub duration: VarInt,
    pub flags: u8,
    pub factor_data: Option<NbtTag<'a>>,
}
