//! Entity lifecycle, movement, metadata and attributes.

use pcore::{
    Decode,
    decoder::{Decoder, NbtTag, Slot, Uuid, VarInt},
    errors::Error,
};

use crate::packets::{counted, optional_chat};

#[derive(Decode, Debug, Clone)]
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

#[derive(Decode, Debug, Clone)]
pub struct SpawnExperienceOrb {
    pub entity_id: VarInt,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub count: i16,
}

#[derive(Decode, Debug, Clone)]
pub struct SpawnPlayer {
    pub entity_id: VarInt,
    pub player_uuid: Uuid,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: u8,
    pub pitch: u8,
}

#[derive(Decode, Debug, Clone)]
pub struct EntityAnimation {
    pub entity_id: VarInt,
    pub animation: u8,
}

#[derive(Decode, Debug, Clone)]
pub struct EntityEvent {
    pub entity_id: i32,
    pub entity_status: i8,
}

#[derive(Decode, Debug, Clone)]
pub struct UpdateEntityPosition {
    pub entity_id: VarInt,
    pub delta_x: i16,
    pub delta_y: i16,
    pub delta_z: i16,
    pub on_ground: bool,
}

#[derive(Decode, Debug, Clone)]
pub struct UpdateEntityPositionAndRotation {
    pub entity_id: VarInt,
    pub delta_x: i16,
    pub delta_y: i16,
    pub delta_z: i16,
    pub yaw: u8,
    pub pitch: u8,
    pub on_ground: bool,
}

#[derive(Decode, Debug, Clone)]
pub struct UpdateEntityRotation {
    pub entity_id: VarInt,
    pub yaw: u8,
    pub pitch: u8,
    pub on_ground: bool,
}

#[derive(Decode, Debug, Clone)]
pub struct MoveVehicle {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Decode, Debug, Clone)]
pub struct LookAt {
    pub feet_eyes: VarInt,
    pub target_x: f64,
    pub target_y: f64,
    pub target_z: f64,
    pub is_entity: bool,
    pub entity_id: Option<VarInt>,
    pub entity_feet_eyes: Option<VarInt>,
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

#[derive(Decode, Debug, Clone)]
pub struct RemoveEntityEffect {
    pub entity_id: VarInt,
    pub effect_id: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct SetHeadRotation {
    pub entity_id: VarInt,
    pub head_yaw: u8,
}

#[derive(Decode, Debug, Clone)]
pub struct SetCamera {
    pub camera_id: VarInt,
}

/// One metadata entry. The value type is chosen by the entry's index, so the
/// payload is kept as a typed enum rather than raw bytes.
#[derive(Debug, Clone)]
pub struct MetadataEntry<'a> {
    pub index: u8,
    pub value: MetadataValue<'a>,
}

/// The metadata value types of protocol 760.
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
    OptBlockId(Option<VarInt>),
    Nbt(NbtTag<'a>),
    Particle,
    VillagerData {
        kind: VarInt,
        profession: VarInt,
        level: VarInt,
    },
    OptVarInt(Option<VarInt>),
    Pose(VarInt),
}

impl<'a> MetadataValue<'a> {
    /// Reads the value for metadata index `index`.
    fn read(decoder: &mut Decoder<'a>, index: u8) -> Result<Self, Error> {
        Ok(match index {
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
            13 => Self::OptBlockId(if decoder.varint()?.get() == 0 {
                None
            } else {
                Some(decoder.varint()?)
            }),
            14 => Self::Nbt(decoder.nbt()?),
            // Index 15 is a particle: its layout depends on the particle id,
            // which this proxy does not model, so it is rejected rather than
            // silently mis-parsed.
            15 => return Err(Error::Nbt("particle metadata is not supported")),
            16 => Self::VillagerData {
                kind: decoder.varint()?,
                profession: decoder.varint()?,
                level: decoder.varint()?,
            },
            17 => Self::OptVarInt(if decoder.bool()? {
                Some(decoder.varint()?)
            } else {
                None
            }),
            18 => Self::Pose(decoder.varint()?),
            other => return Err(Error::UnknownOpcode(other as i32)),
        })
    }
}

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
            let value = MetadataValue::read(decoder, index)?;
            metadata.push(MetadataEntry { index, value });
        }
        Ok(Self {
            entity_id,
            metadata,
        })
    }
}

#[derive(Decode, Debug, Clone)]
pub struct LinkEntities {
    pub attached_entity_id: i32,
    pub holding_entity_id: i32,
}

#[derive(Decode, Debug, Clone)]
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
        let entity_id = decoder.varint()?;
        let mut equipment = Vec::new();
        // The high bit of the slot byte means "another entry follows".
        loop {
            let raw = decoder.i8()?;
            let slot = raw & 0x7F;
            let item = Slot::decode(decoder)?;
            equipment.push(EquipmentEntry { slot, item });
            if raw & 0x80u8 as i8 == 0 {
                break;
            }
        }
        Ok(Self {
            entity_id,
            equipment,
        })
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

#[derive(Decode, Debug, Clone)]
pub struct PickupItem {
    pub collected_entity_id: VarInt,
    pub collector_entity_id: VarInt,
    pub pickup_item_count: VarInt,
}

#[derive(Decode, Debug, Clone)]
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

#[derive(Decode, Debug, Clone)]
pub struct EntityEffect<'a> {
    pub entity_id: VarInt,
    pub effect_id: VarInt,
    pub amplifier: i8,
    pub duration: VarInt,
    pub flags: u8,
    pub has_factor_data: bool,
    pub factor_data: Option<NbtTag<'a>>,
}
