//! Scoreboard objectives, teams and scores.

use pcore::{
    Decode, Encode,
    decoder::{Decoder, MAX_ELEMENTS, VarInt},
    encoder::Encoder,
    errors::Error,
};

#[derive(Decode, Encode, Debug, Clone)]
pub struct DisplayObjective<'a> {
    pub position: u8,
    pub score_name: &'a str,
}

/// `Update Objectives` (0x56).
///
/// Hand-written `Decode`: `objective_value` and `kind` are present for modes 0
/// (create) and 2 (update text), not gated by their own booleans.
#[derive(Debug, Clone)]
pub struct UpdateObjectives<'a> {
    pub objective_name: &'a str,
    pub mode: u8,
    pub objective_value: Option<&'a str>,
    /// 0 = integer, 1 = hearts.
    pub kind: Option<VarInt>,
}

impl<'a> Decode<'a> for UpdateObjectives<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let objective_name = decoder.string()?;
        let mode = decoder.u8()?;
        let (objective_value, kind) = if mode == 0 || mode == 2 {
            (Some(decoder.string()?), Some(decoder.varint()?))
        } else {
            (None, None)
        };
        Ok(Self {
            objective_name,
            mode,
            objective_value,
            kind,
        })
    }
}

impl Encode for UpdateObjectives<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.string(self.objective_name);
        encoder.u8(self.mode);
        if self.mode == 0 || self.mode == 2 {
            encoder.string(self.objective_value.unwrap_or(""));
            encoder.varint(self.kind.map(|k| k.get()).unwrap_or(0));
        }
    }
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
                entities: team_entities(decoder)?,
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
                entities: team_entities(decoder)?,
            },
            4 => TeamAction::RemoveEntities {
                entities: team_entities(decoder)?,
            },
            other => return Err(Error::UnknownOpcode(other as i32)),
        };
        Ok(Self { team_name, action })
    }
}

/// Reads the `Array of String (40)` of a team.
///
/// `String (40)` is a protocol String — VarInt length plus a 40-character
/// limit — not the NBT `u16` framing.
fn team_entities<'a>(decoder: &mut Decoder<'a>) -> Result<Vec<&'a str>, Error> {
    let len = decoder.varint()?.get();
    if len < 0 {
        return Err(Error::Read(0));
    }
    let len = len as usize;
    if len > MAX_ELEMENTS {
        return Err(Error::LenLimit(len));
    }
    let mut entities = Vec::with_capacity(len);
    for _ in 0..len {
        entities.push(decoder.string()?);
    }
    Ok(entities)
}

impl Encode for TeamAction<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        match self {
            Self::Create {
                display_name,
                friendly_flags,
                name_tag_visibility,
                collision_rule,
                color,
                prefix,
                suffix,
                entities,
            } => {
                encoder.u8(0);
                encoder.string(display_name);
                encoder.u8(*friendly_flags);
                encoder.string(name_tag_visibility);
                encoder.string(collision_rule);
                encoder.varint(color.get());
                encoder.string(prefix);
                encoder.string(suffix);
                encoder.encode(entities);
            }
            Self::Remove => encoder.u8(1),
            Self::Update {
                display_name,
                friendly_flags,
                name_tag_visibility,
                collision_rule,
                color,
                prefix,
                suffix,
            } => {
                encoder.u8(2);
                encoder.string(display_name);
                encoder.u8(*friendly_flags);
                encoder.string(name_tag_visibility);
                encoder.string(collision_rule);
                encoder.varint(color.get());
                encoder.string(prefix);
                encoder.string(suffix);
            }
            Self::AddEntities { entities } => {
                encoder.u8(3);
                encoder.encode(entities);
            }
            Self::RemoveEntities { entities } => {
                encoder.u8(4);
                encoder.encode(entities);
            }
        }
    }
}

impl Encode for UpdateTeams<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.string(self.team_name);
        self.action.encode(encoder);
    }
}

/// `Update Score` (0x59).
///
/// Hand-written `Decode`: `value` is present for every action except 1
/// (remove), not gated by its own boolean.
#[derive(Debug, Clone)]
pub struct UpdateScore<'a> {
    pub entity_name: &'a str,
    /// 0 = create/update, 1 = remove.
    pub action: VarInt,
    pub objective_name: &'a str,
    pub value: Option<VarInt>,
}

impl<'a> Decode<'a> for UpdateScore<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let entity_name = decoder.string()?;
        let action = decoder.varint()?;
        let objective_name = decoder.string()?;
        let value = if action.get() != 1 {
            Some(decoder.varint()?)
        } else {
            None
        };
        Ok(Self {
            entity_name,
            action,
            objective_name,
            value,
        })
    }
}

impl Encode for UpdateScore<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.string(self.entity_name);
        encoder.varint(self.action.get());
        encoder.string(self.objective_name);
        if self.action.get() != 1 {
            encoder.varint(self.value.map(|v| v.get()).unwrap_or(0));
        }
    }
}
