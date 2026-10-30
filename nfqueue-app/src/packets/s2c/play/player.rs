//! Player state: join/respawn, abilities, health, experience, player list and
//! combat.

use pcore::{
    Decode,
    decoder::{Decoder, NbtTag, Uuid, VarInt},
    errors::Error,
};

use crate::packets::{counted, optional_chat};

/// `statistics` is `count` entries of `(category, statistic, value)`, not a
/// byte array — the count is an element count, so the derive macro cannot
/// express it.
#[derive(Debug, Clone)]
pub struct AwardStatistics<'a> {
    pub statistics: Vec<Statistic<'a>>,
}

#[derive(Debug, Clone)]
pub struct Statistic<'a> {
    pub category_id: VarInt,
    pub statistic_id: VarInt,
    pub value: VarInt,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> Decode<'a> for AwardStatistics<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let count = decoder.varint()?.get();
        if count < 0 {
            return Err(Error::Nbt("negative statistic count"));
        }
        let mut statistics = Vec::with_capacity(count as usize);
        for _ in 0..count {
            statistics.push(Statistic {
                category_id: decoder.varint()?,
                statistic_id: decoder.varint()?,
                value: decoder.varint()?,
                _marker: std::marker::PhantomData,
            });
        }
        Ok(Self { statistics })
    }
}

#[derive(Decode, Debug, Clone)]
pub struct ChangeDifficulty {
    pub difficulty: u8,
    pub locked: bool,
}

/// Join-game packet. Hand-written `Decode` because `dimension_names` is a
/// VarInt **count** of Identifiers, not a byte array.
#[derive(Debug, Clone)]
pub struct Login<'a> {
    pub entity_id: i32,
    pub is_hardcore: bool,
    pub gamemode: u8,
    pub previous_gamemode: i8,
    pub dimension_names: Vec<&'a str>,
    pub registry_codec: NbtTag<'a>,
    pub dimension_type: &'a str,
    pub dimension_name: &'a str,
    pub hashed_seed: i64,
    pub max_players: VarInt,
    pub view_distance: VarInt,
    pub simulation_distance: VarInt,
    pub reduced_debug_info: bool,
    pub enable_respawn_screen: bool,
    pub is_debug: bool,
    pub is_flat: bool,
    pub death_dimension_name: Option<&'a str>,
    pub death_location: Option<u64>,
}

impl<'a> Decode<'a> for Login<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let entity_id = decoder.i32()?;
        let is_hardcore = decoder.bool()?;
        let gamemode = decoder.u8()?;
        let previous_gamemode = decoder.i8()?;

        let dimension_count = decoder.varint()?.get();
        if dimension_count < 0 {
            return Err(Error::Nbt("negative dimension count"));
        }
        let mut dimension_names = Vec::with_capacity(dimension_count as usize);
        for _ in 0..dimension_count {
            dimension_names.push(decoder.string()?);
        }

        let registry_codec = decoder.nbt()?;
        let dimension_type = decoder.string()?;
        let dimension_name = decoder.string()?;
        let hashed_seed = decoder.i64()?;
        let max_players = decoder.varint()?;
        let view_distance = decoder.varint()?;
        let simulation_distance = decoder.varint()?;
        let reduced_debug_info = decoder.bool()?;
        let enable_respawn_screen = decoder.bool()?;
        let is_debug = decoder.bool()?;
        let is_flat = decoder.bool()?;

        let has_death_location = decoder.bool()?;
        let (death_dimension_name, death_location) = if has_death_location {
            (Some(decoder.string()?), Some(decoder.u64()?))
        } else {
            (None, None)
        };

        Ok(Self {
            entity_id,
            is_hardcore,
            gamemode,
            previous_gamemode,
            dimension_names,
            registry_codec,
            dimension_type,
            dimension_name,
            hashed_seed,
            max_players,
            view_distance,
            simulation_distance,
            reduced_debug_info,
            enable_respawn_screen,
            is_debug,
            is_flat,
            death_dimension_name,
            death_location,
        })
    }
}

/// `Respawn` (0x3E).
///
/// Hand-written `Decode`: the death location pair is gated by
/// `has_death_location`, not by a per-field boolean.
#[derive(Debug, Clone)]
pub struct Respawn<'a> {
    pub dimension_type: &'a str,
    pub dimension_name: &'a str,
    pub hashed_seed: i64,
    pub gamemode: u8,
    pub previous_gamemode: i8,
    pub is_debug: bool,
    pub is_flat: bool,
    pub copy_metadata: bool,
    pub has_death_location: bool,
    pub death_dimension_name: Option<&'a str>,
    pub death_location: Option<u64>,
}

impl<'a> Decode<'a> for Respawn<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let dimension_type = decoder.string()?;
        let dimension_name = decoder.string()?;
        let hashed_seed = decoder.i64()?;
        let gamemode = decoder.u8()?;
        let previous_gamemode = decoder.i8()?;
        let is_debug = decoder.bool()?;
        let is_flat = decoder.bool()?;
        let copy_metadata = decoder.bool()?;
        let has_death_location = decoder.bool()?;
        let (death_dimension_name, death_location) = if has_death_location {
            (Some(decoder.string()?), Some(decoder.u64()?))
        } else {
            (None, None)
        };
        Ok(Self {
            dimension_type,
            dimension_name,
            hashed_seed,
            gamemode,
            previous_gamemode,
            is_debug,
            is_flat,
            copy_metadata,
            has_death_location,
            death_dimension_name,
            death_location,
        })
    }
}

#[derive(Decode, Debug, Clone)]
pub struct PlayerAbilities {
    pub flags: u8,
    pub flying_speed: f32,
    pub field_of_view_modifier: f32,
}

#[derive(Decode, Debug, Clone)]
pub struct SynchronizePlayerPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub flags: u8,
    pub teleport_id: VarInt,
    pub dismount_vehicle: bool,
}

#[derive(Decode, Debug, Clone)]
pub struct SetExperience {
    pub experience_bar: f32,
    pub level: VarInt,
    pub total_experience: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct SetHealth {
    pub health: f32,
    pub food: VarInt,
    pub food_saturation: f32,
}

#[derive(Decode, Debug, Clone)]
pub struct SetDefaultSpawnPosition {
    pub location: u64,
    pub angle: f32,
}

#[derive(Decode, Debug, Clone)]
pub struct SetDisplayChatPreview {
    pub chat_preview_setting: bool,
}

#[derive(Decode, Debug, Clone)]
pub struct SetRenderDistance {
    pub view_distance: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct SetSimulationDistance {
    pub simulation_distance: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct EndCombat {
    pub duration: VarInt,
    pub entity_id: i32,
}

#[derive(Decode, Debug, Clone)]
pub struct EnterCombat {}

#[derive(Decode, Debug, Clone)]
pub struct CombatDeath<'a> {
    pub player_id: VarInt,
    pub entity_id: i32,
    pub message: &'a str,
}

/// `Player Info` action, protocol 760.
///
/// In 1.19.2 this is a single enum, not a bitmask: the action selects exactly
/// one body layout for every entry in the array.
pub const PLAYER_INFO_ADD_PLAYER: i32 = 0;
pub const PLAYER_INFO_UPDATE_GAME_MODE: i32 = 1;
pub const PLAYER_INFO_UPDATE_LATENCY: i32 = 2;
pub const PLAYER_INFO_UPDATE_DISPLAY_NAME: i32 = 3;
pub const PLAYER_INFO_REMOVE_PLAYER: i32 = 4;

/// A signed skin/cape property.
#[derive(Debug, Clone)]
pub struct PlayerProperty<'a> {
    pub name: &'a str,
    pub value: &'a str,
    pub signature: Option<&'a str>,
}

/// The chat-initialisation block of an `add player` entry.
///
/// It is opaque key material this proxy never inspects, but it still has to be
/// consumed to keep the cursor in sync.
#[derive(Debug, Clone)]
pub struct PlayerSignatureData<'a> {
    pub timestamp: i64,
    pub public_key: &'a [u8],
    pub signature: &'a [u8],
}

/// One player entry of `Player Info`. Which fields are populated depends on the
/// packet's `action`, so absent fields stay `None`.
#[derive(Debug, Clone)]
pub struct PlayerInfoEntry<'a> {
    pub uuid: Uuid,
    pub name: Option<&'a str>,
    pub properties: Option<Vec<PlayerProperty<'a>>>,
    pub game_mode: Option<VarInt>,
    pub latency: Option<VarInt>,
    pub display_name: Option<Option<&'a str>>,
    pub signature_data: Option<PlayerSignatureData<'a>>,
}

/// `Player Info` (0x37).
#[derive(Debug, Clone)]
pub struct PlayerInfo<'a> {
    pub action: VarInt,
    pub players: Vec<PlayerInfoEntry<'a>>,
}

impl<'a> Decode<'a> for PlayerInfo<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let action = decoder.varint()?;
        let action_id = action.get();

        let players = counted(decoder, |d| {
            let uuid = d.uuid()?;
            let mut entry = PlayerInfoEntry {
                uuid,
                name: None,
                properties: None,
                game_mode: None,
                latency: None,
                display_name: None,
                signature_data: None,
            };

            match action_id {
                PLAYER_INFO_ADD_PLAYER => {
                    entry.name = Some(d.string()?);
                    entry.properties = Some(counted(d, |d| {
                        Ok(PlayerProperty {
                            name: d.string()?,
                            value: d.string()?,
                            signature: if d.bool()? { Some(d.string()?) } else { None },
                        })
                    })?);
                    entry.game_mode = Some(d.varint()?);
                    entry.latency = Some(d.varint()?);
                    entry.display_name = Some(optional_chat(d)?);
                    // `has sig data` gates the key material.
                    if d.bool()? {
                        entry.signature_data = Some(PlayerSignatureData {
                            timestamp: d.i64()?,
                            public_key: d.slice()?,
                            signature: d.slice()?,
                        });
                    }
                }
                PLAYER_INFO_UPDATE_GAME_MODE => entry.game_mode = Some(d.varint()?),
                PLAYER_INFO_UPDATE_LATENCY => entry.latency = Some(d.varint()?),
                PLAYER_INFO_UPDATE_DISPLAY_NAME => entry.display_name = Some(optional_chat(d)?),
                PLAYER_INFO_REMOVE_PLAYER => {}
                other => return Err(Error::UnknownOpcode(other)),
            }

            Ok(entry)
        })?;
        Ok(Self { action, players })
    }
}
