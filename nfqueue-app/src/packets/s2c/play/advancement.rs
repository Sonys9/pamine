//! Advancements, recipes and tags.

use pcore::{
    Decode,
    decoder::{Decoder, Slot, VarInt},
    errors::Error,
};

use crate::packets::{counted, identifiers};

/// One advancement's progress.
#[derive(Debug, Clone)]
pub struct AdvancementProgress<'a> {
    pub id: &'a str,
    /// `None` when the advancement has no parent.
    pub parent_id: Option<&'a str>,
    /// `None` when the advancement has no display data.
    pub display: Option<AdvancementDisplay<'a>>,
    pub criteria: Vec<&'a str>,
}

/// The display block of an advancement.
#[derive(Debug, Clone)]
pub struct AdvancementDisplay<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub icon: Slot<'a>,
    pub frame_type: VarInt,
    pub flags: i32,
    /// Only present when the `0x01` flag (has background) is set.
    pub background_texture: Option<&'a str>,
    pub x_coord: f32,
    pub y_coord: f32,
}

/// One advancement that was removed or whose progress changed.
#[derive(Debug, Clone)]
pub struct AdvancementRemoval<'a> {
    pub id: &'a str,
}

#[derive(Debug, Clone)]
pub struct UpdateAdvancements<'a> {
    pub reset: bool,
    pub added: Vec<AdvancementProgress<'a>>,
    pub removed: Vec<&'a str>,
    pub progress: Vec<AdvancementProgressUpdate<'a>>,
}

/// A progress-only update: the advancement id plus its completed criteria.
#[derive(Debug, Clone)]
pub struct AdvancementProgressUpdate<'a> {
    pub id: &'a str,
    pub criteria: Vec<&'a str>,
}

impl<'a> Decode<'a> for UpdateAdvancements<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let reset = decoder.bool()?;

        let added = counted(decoder, |d| {
            let id = d.string()?;
            let parent_id = if d.bool()? { Some(d.string()?) } else { None };
            let display = if d.bool()? {
                let title = d.string()?;
                let description = d.string()?;
                let icon = Slot::decode(d)?;
                let frame_type = d.varint()?;
                let flags = d.i32()?;
                // Bit 0 of `flags` marks a background texture.
                let background_texture = if flags & 0x01 != 0 {
                    Some(d.string()?)
                } else {
                    None
                };
                let x_coord = d.f32()?;
                let y_coord = d.f32()?;
                Some(AdvancementDisplay {
                    title,
                    description,
                    icon,
                    frame_type,
                    flags,
                    background_texture,
                    x_coord,
                    y_coord,
                })
            } else {
                None
            };
            let criteria = identifiers(d)?;
            Ok(AdvancementProgress {
                id,
                parent_id,
                display,
                criteria,
            })
        })?;

        let removed = identifiers(decoder)?;

        let progress = counted(decoder, |d| {
            Ok(AdvancementProgressUpdate {
                id: d.string()?,
                criteria: identifiers(d)?,
            })
        })?;

        Ok(Self {
            reset,
            added,
            removed,
            progress,
        })
    }
}

#[derive(Decode, Debug, Clone)]
pub struct SelectAdvancementsTab<'a> {
    pub has_id: bool,
    pub tab_id: Option<&'a str>,
}

/// One recipe: an identifier plus an opaque, type-dependent payload.
#[derive(Debug, Clone)]
pub struct Recipe<'a> {
    pub id: &'a str,
    /// The recipe body. Its layout depends on the recipe `type`, which is a
    /// registry entry this proxy does not model, so the bytes are kept as-is.
    pub data: &'a [u8],
}

#[derive(Debug, Clone)]
pub struct UpdateRecipes<'a> {
    pub recipes: Vec<Recipe<'a>>,
}

impl<'a> Decode<'a> for UpdateRecipes<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let recipes = counted(decoder, |d| {
            let id = d.string()?;
            // The recipe type is a VarInt registry id; the body that follows
            // has no length prefix, so it cannot be delimited without knowing
            // every recipe type. The whole packet is therefore consumed here.
            let _recipe_type = d.varint()?;
            Ok(Recipe { id, data: d.rest() })
        })?;
        Ok(Self { recipes })
    }
}

/// One named tag group: an identifier and the entries it contains.
#[derive(Debug, Clone)]
pub struct TagGroup<'a> {
    pub name: &'a str,
    pub entries: Vec<VarInt>,
}

/// All tag groups of one registry type.
#[derive(Debug, Clone)]
pub struct TagRegistry<'a> {
    pub registry: &'a str,
    pub tags: Vec<TagGroup<'a>>,
}

#[derive(Debug, Clone)]
pub struct UpdateTags<'a> {
    pub registries: Vec<TagRegistry<'a>>,
}

impl<'a> Decode<'a> for UpdateTags<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let registries = counted(decoder, |d| {
            Ok(TagRegistry {
                registry: d.string()?,
                tags: counted(d, |d| {
                    Ok(TagGroup {
                        name: d.string()?,
                        entries: counted(d, |d| d.varint())?,
                    })
                })?,
            })
        })?;
        Ok(Self { registries })
    }
}
