//! Titles, boss bars and other on-screen UI elements.

use pcore::{
    Decode,
    decoder::{Decoder, Uuid, VarInt},
    errors::Error,
};

/// The `action`-dependent body of [`BossBar`].
#[derive(Debug, Clone)]
pub enum BossBarAction<'a> {
    /// `0` — a new bar appears.
    Add {
        title: &'a str,
        health: f32,
        color: VarInt,
        division: VarInt,
        flags: u8,
    },
    /// `1` — the bar disappears; nothing follows.
    Remove,
    /// `2` — only the health changed.
    UpdateHealth { health: f32 },
    /// `3` — only the title changed.
    UpdateTitle { title: &'a str },
    /// `4` — only the colour and division changed.
    UpdateStyle { color: VarInt, division: VarInt },
    /// `5` — only the flags changed.
    UpdateFlags { flags: u8 },
}

#[derive(Debug, Clone)]
pub struct BossBar<'a> {
    pub uuid: Uuid,
    pub action: BossBarAction<'a>,
}

impl<'a> Decode<'a> for BossBar<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let uuid = decoder.uuid()?;
        let action = match decoder.varint()?.get() {
            0 => BossBarAction::Add {
                title: decoder.string()?,
                health: decoder.f32()?,
                color: decoder.varint()?,
                division: decoder.varint()?,
                flags: decoder.u8()?,
            },
            1 => BossBarAction::Remove,
            2 => BossBarAction::UpdateHealth {
                health: decoder.f32()?,
            },
            3 => BossBarAction::UpdateTitle {
                title: decoder.string()?,
            },
            4 => BossBarAction::UpdateStyle {
                color: decoder.varint()?,
                division: decoder.varint()?,
            },
            5 => BossBarAction::UpdateFlags {
                flags: decoder.u8()?,
            },
            other => return Err(Error::UnknownOpcode(other)),
        };
        Ok(Self { uuid, action })
    }
}

#[derive(Decode, Debug, Clone)]
pub struct ClearTitles {
    pub reset: bool,
}

#[derive(Decode, Debug, Clone)]
pub struct SetActionBarText<'a> {
    pub action_bar_text: &'a str,
}

#[derive(Decode, Debug, Clone)]
pub struct SetSubtitleText<'a> {
    pub subtitle_text: &'a str,
}

#[derive(Decode, Debug, Clone)]
pub struct SetTitleText<'a> {
    pub title_text: &'a str,
}

#[derive(Decode, Debug, Clone)]
pub struct SetTitleAnimationTimes {
    pub fade_in: i32,
    pub stay: i32,
    pub fade_out: i32,
}
