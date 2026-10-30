//! Scoreboard objectives, teams and scores.

use pcore::{
    Decode,
    decoder::{Decoder, VarInt},
    errors::Error,
};

use crate::packets::identifiers;

#[derive(Decode, Debug, Clone)]
pub struct DisplayObjective<'a> {
    pub position: u8,
    pub score_name: &'a str,
}

#[derive(Decode, Debug, Clone)]
pub struct UpdateObjectives<'a> {
    pub objective_name: &'a str,
    pub mode: u8,
    pub objective_value: Option<&'a str>,
    pub kind: Option<VarInt>,
}

/// The `mode`-dependent body of [`UpdateTeams`].
#[derive(Debug, Clone)]
pub enum TeamAction<'a> {
    /// `0` — a new team is created.
    Create {
        display_name: &'a str,
        friendly_flags: u8,
        name_tag_visibility: &'a str,
        collision_rule: &'a str,
        color: VarInt,
        prefix: &'a str,
        suffix: &'a str,
        entities: Vec<&'a str>,
    },
    /// `1` — the team is removed; nothing follows.
    Remove,
    /// `2` — only the display metadata changed.
    Update {
        display_name: &'a str,
        friendly_flags: u8,
        name_tag_visibility: &'a str,
        collision_rule: &'a str,
        color: VarInt,
        prefix: &'a str,
        suffix: &'a str,
    },
    /// `3` — entities joined the team.
    AddEntities { entities: Vec<&'a str> },
    /// `4` — entities left the team.
    RemoveEntities { entities: Vec<&'a str> },
}

#[derive(Debug, Clone)]
pub struct UpdateTeams<'a> {
    pub team_name: &'a str,
    pub action: TeamAction<'a>,
}

impl<'a> Decode<'a> for UpdateTeams<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let team_name = decoder.string()?;
        let action = match decoder.u8()? {
            0 => TeamAction::Create {
                display_name: decoder.string()?,
                friendly_flags: decoder.u8()?,
                name_tag_visibility: decoder.string()?,
                collision_rule: decoder.string()?,
                color: decoder.varint()?,
                prefix: decoder.string()?,
                suffix: decoder.string()?,
                entities: identifiers(decoder)?,
            },
            1 => TeamAction::Remove,
            2 => TeamAction::Update {
                display_name: decoder.string()?,
                friendly_flags: decoder.u8()?,
                name_tag_visibility: decoder.string()?,
                collision_rule: decoder.string()?,
                color: decoder.varint()?,
                prefix: decoder.string()?,
                suffix: decoder.string()?,
            },
            3 => TeamAction::AddEntities {
                entities: identifiers(decoder)?,
            },
            4 => TeamAction::RemoveEntities {
                entities: identifiers(decoder)?,
            },
            other => return Err(Error::UnknownOpcode(other as i32)),
        };
        Ok(Self { team_name, action })
    }
}

#[derive(Decode, Debug, Clone)]
pub struct UpdateScore<'a> {
    pub entity_name: &'a str,
    pub action: VarInt,
    pub objective_name: &'a str,
    pub value: Option<VarInt>,
}
