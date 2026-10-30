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

#[derive(Decode, Debug, Clone)]
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

/// Bit flags of the `Player Info` action mask.
pub const PLAYER_INFO_ADD_PLAYER: u8 = 0x01;
pub const PLAYER_INFO_INITIALIZE_CHAT: u8 = 0x02;
pub const PLAYER_INFO_UPDATE_GAME_MODE: u8 = 0x04;
pub const PLAYER_INFO_UPDATE_LISTED: u8 = 0x08;
pub const PLAYER_INFO_UPDATE_LATENCY: u8 = 0x10;
pub const PLAYER_INFO_UPDATE_DISPLAY_NAME: u8 = 0x20;

/// A signed skin/cape property.
#[derive(Debug, Clone)]
pub struct PlayerProperty<'a> {
    pub name: &'a str,
    pub value: &'a str,
    pub signature: Option<&'a str>,
}

/// One player entry of `Player Info`. Every field past `uuid` is gated by the
/// packet's `actions` bitmask, so absent fields stay `None`.
#[derive(Debug, Clone)]
pub struct PlayerInfoEntry<'a> {
    pub uuid: Uuid,
    pub name: Option<&'a str>,
    pub properties: Option<Vec<PlayerProperty<'a>>>,
    pub game_mode: Option<VarInt>,
    pub listed: Option<bool>,
    pub latency: Option<VarInt>,
    pub display_name: Option<Option<&'a str>>,
}

#[derive(Debug, Clone)]
pub struct PlayerInfo<'a> {
    pub actions: u8,
    pub players: Vec<PlayerInfoEntry<'a>>,
}

impl<'a> Decode<'a> for PlayerInfo<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let actions = decoder.u8()?;
        let players = counted(decoder, |d| {
            let uuid = d.uuid()?;

            let (name, properties) = if actions & PLAYER_INFO_ADD_PLAYER != 0 {
                let name = d.string()?;
                let properties = counted(d, |d| {
                    Ok(PlayerProperty {
                        name: d.string()?,
                        value: d.string()?,
                        signature: if d.bool()? { Some(d.string()?) } else { None },
                    })
                })?;
                (Some(name), Some(properties))
            } else {
                (None, None)
            };

            // The chat-initialisation block (session id + key signature) is
            // skipped: it is opaque key material this proxy never inspects.
            if actions & PLAYER_INFO_INITIALIZE_CHAT != 0 {
                let _has_signature = d.bool()?;
                let _session_id = d.uuid()?;
                let _key_signature = d.slice()?;
            }

            let game_mode = if actions & PLAYER_INFO_UPDATE_GAME_MODE != 0 {
                Some(d.varint()?)
            } else {
                None
            };
            let listed = if actions & PLAYER_INFO_UPDATE_LISTED != 0 {
                Some(d.bool()?)
            } else {
                None
            };
            let latency = if actions & PLAYER_INFO_UPDATE_LATENCY != 0 {
                Some(d.varint()?)
            } else {
                None
            };
            let display_name = if actions & PLAYER_INFO_UPDATE_DISPLAY_NAME != 0 {
                Some(optional_chat(d)?)
            } else {
                None
            };

            Ok(PlayerInfoEntry {
                uuid,
                name,
                properties,
                game_mode,
                listed,
                latency,
                display_name,
            })
        })?;
        Ok(Self { actions, players })
    }
}
