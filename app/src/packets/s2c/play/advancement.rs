//! Advancements, recipes and tags.

use pcore::{
    Decode, Encode,
    decoder::{Decoder, Slot, VarInt},
    encoder::Encoder,
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
    /// Criterion identifiers. Each carries no payload on the wire.
    pub criteria: Vec<&'a str>,
    /// Requirement groups: every inner list is a set of criteria that must all
    /// be met, and the outer list is satisfied when any group is.
    pub requirements: Vec<Vec<&'a str>>,
}

/// The display block of an advancement.
#[derive(Debug, Clone)]
pub struct AdvancementDisplay<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub icon: Slot<'a>,
    /// 0 = task, 1 = challenge, 2 = goal.
    pub frame_type: VarInt,
    /// 0x01: has background texture, 0x02: show toast, 0x04: hidden.
    pub flags: i32,
    /// Only present when the `0x01` flag (has background) is set.
    pub background_texture: Option<&'a str>,
    pub x_coord: f32,
    pub y_coord: f32,
}

/// One criterion of an advancement and whether it has been achieved.
#[derive(Debug, Clone)]
pub struct CriterionProgress<'a> {
    pub id: &'a str,
    /// `None` when the criterion has not been achieved.
    pub achieved_at: Option<i64>,
}

/// A progress-only update: the advancement id plus its criteria states.
#[derive(Debug, Clone)]
pub struct AdvancementProgressUpdate<'a> {
    pub id: &'a str,
    pub criteria: Vec<CriterionProgress<'a>>,
}

/// `Update Advancements` (0x67).
///
/// Hand-written `Decode`: the advancement body is a chain of gated sections
/// (parent, display, criteria, requirements) and the progress entries carry a
/// per-criterion `achieved` gate.
#[derive(Debug, Clone)]
pub struct UpdateAdvancements<'a> {
    pub reset: bool,
    pub added: Vec<AdvancementProgress<'a>>,
    pub removed: Vec<&'a str>,
    pub progress: Vec<AdvancementProgressUpdate<'a>>,
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
            // Criteria are identifiers with no payload.
            let criteria = identifiers(d)?;
            // Requirements: a count of groups, each a count of identifiers.
            let requirements = counted(d, |d| identifiers(d))?;
            Ok(AdvancementProgress {
                id,
                parent_id,
                display,
                criteria,
                requirements,
            })
        })?;

        let removed = identifiers(decoder)?;

        let progress = counted(decoder, |d| {
            let id = d.string()?;
            let criteria = counted(d, |d| {
                let id = d.string()?;
                let achieved_at = if d.bool()? { Some(d.i64()?) } else { None };
                Ok(CriterionProgress { id, achieved_at })
            })?;
            Ok(AdvancementProgressUpdate { id, criteria })
        })?;

        Ok(Self {
            reset,
            added,
            removed,
            progress,
        })
    }
}

impl Encode for AdvancementDisplay<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.string(self.title);
        encoder.string(self.description);
        encoder.slot(&self.icon);
        encoder.varint(self.frame_type.get());
        encoder.i32(self.flags);
        if self.flags & 0x01 != 0 {
            encoder.string(self.background_texture.unwrap_or(""));
        }
        encoder.f32(self.x_coord);
        encoder.f32(self.y_coord);
    }
}

impl Encode for AdvancementProgress<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.string(self.id);
        match self.parent_id {
            Some(parent) => {
                encoder.bool(true);
                encoder.string(parent);
            }
            None => encoder.bool(false),
        }
        match &self.display {
            Some(display) => {
                encoder.bool(true);
                display.encode(encoder);
            }
            None => encoder.bool(false),
        }
        encoder.encode(&self.criteria);
        encoder.varint(self.requirements.len() as i32);
        for group in &self.requirements {
            encoder.encode(group);
        }
    }
}

impl Encode for CriterionProgress<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.string(self.id);
        match self.achieved_at {
            Some(at) => {
                encoder.bool(true);
                encoder.i64(at);
            }
            None => encoder.bool(false),
        }
    }
}

impl Encode for AdvancementProgressUpdate<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.string(self.id);
        encoder.varint(self.criteria.len() as i32);
        for criterion in &self.criteria {
            criterion.encode(encoder);
        }
    }
}

impl Encode for UpdateAdvancements<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.bool(self.reset);
        encoder.varint(self.added.len() as i32);
        for advancement in &self.added {
            advancement.encode(encoder);
        }
        encoder.encode(&self.removed);
        encoder.varint(self.progress.len() as i32);
        for update in &self.progress {
            update.encode(encoder);
        }
    }
}

#[derive(Debug, Clone)]
pub struct SelectAdvancementsTab<'a> {
    pub has_id: bool,
    pub tab_id: Option<&'a str>,
}

impl<'a> Decode<'a> for SelectAdvancementsTab<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let has_id = decoder.bool()?;
        let tab_id = if has_id {
            Some(decoder.string()?)
        } else {
            None
        };
        Ok(Self { has_id, tab_id })
    }
}

impl Encode for SelectAdvancementsTab<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.bool(self.has_id);
        if self.has_id {
            encoder.string(self.tab_id.unwrap_or(""));
        }
    }
}

/// One ingredient of a recipe: any of `items` may be used.
#[derive(Debug, Clone)]
pub struct Ingredient<'a> {
    pub items: Vec<Slot<'a>>,
}

impl<'a> Decode<'a> for Ingredient<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let items = counted(decoder, |d| Slot::decode(d))?;
        Ok(Self { items })
    }
}

impl Encode for Ingredient<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.varint(self.items.len() as i32);
        for item in &self.items {
            encoder.slot(item);
        }
    }
}

/// The type-dependent body of a recipe.
#[derive(Debug, Clone)]
pub enum RecipeData<'a> {
    /// `crafting_shapeless` — ingredients in any order.
    Shapeless {
        group: &'a str,
        ingredients: Vec<Ingredient<'a>>,
        result: Slot<'a>,
    },
    /// `crafting_shaped` — ingredients laid out in a `width * height` grid.
    Shaped {
        width: VarInt,
        height: VarInt,
        group: &'a str,
        /// `width * height` entries, indexed by `x + y * width`.
        ingredients: Vec<Ingredient<'a>>,
        result: Slot<'a>,
    },
    /// `smelting`, `blasting`, `smoking`, `campfire_cooking`.
    Cooking {
        group: &'a str,
        ingredient: Ingredient<'a>,
        result: Slot<'a>,
        experience: f32,
        cooking_time: VarInt,
    },
    /// `stonecutting`.
    Stonecutting {
        group: &'a str,
        ingredient: Ingredient<'a>,
        result: Slot<'a>,
    },
    /// `smithing`.
    Smithing {
        base: Ingredient<'a>,
        addition: Ingredient<'a>,
        result: Slot<'a>,
    },
    /// Every `crafting_special_*` recipe carries no body.
    Special,
}

/// One recipe: a type, an identifier and a type-dependent body.
#[derive(Debug, Clone)]
pub struct Recipe<'a> {
    /// The recipe type identifier, e.g. `minecraft:crafting_shaped`.
    pub kind: &'a str,
    pub id: &'a str,
    pub data: RecipeData<'a>,
}

/// `Update Recipes` (0x6A).
///
/// Hand-written `Decode`: the wire order is **type first, then id**, and the
/// body that follows has no length prefix, so it can only be delimited by
/// knowing the layout of every recipe type.
#[derive(Debug, Clone)]
pub struct UpdateRecipes<'a> {
    pub recipes: Vec<Recipe<'a>>,
}

impl<'a> Decode<'a> for UpdateRecipes<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let recipes = counted(decoder, |d| {
            let kind = d.string()?;
            let id = d.string()?;
            let data = recipe_data(d, kind)?;
            Ok(Recipe { kind, id, data })
        })?;
        Ok(Self { recipes })
    }
}

impl Encode for RecipeData<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        match self {
            Self::Shapeless {
                group,
                ingredients,
                result,
            } => {
                encoder.string(group);
                encoder.varint(ingredients.len() as i32);
                for ingredient in ingredients {
                    ingredient.encode(encoder);
                }
                encoder.slot(result);
            }
            Self::Shaped {
                width,
                height,
                group,
                ingredients,
                result,
            } => {
                encoder.varint(width.get());
                encoder.varint(height.get());
                encoder.string(group);
                // The grid is `width * height` entries, not a counted array.
                for ingredient in ingredients {
                    ingredient.encode(encoder);
                }
                encoder.slot(result);
            }
            Self::Cooking {
                group,
                ingredient,
                result,
                experience,
                cooking_time,
            } => {
                encoder.string(group);
                ingredient.encode(encoder);
                encoder.slot(result);
                encoder.f32(*experience);
                encoder.varint(cooking_time.get());
            }
            Self::Stonecutting {
                group,
                ingredient,
                result,
            } => {
                encoder.string(group);
                ingredient.encode(encoder);
                encoder.slot(result);
            }
            Self::Smithing {
                base,
                addition,
                result,
            } => {
                base.encode(encoder);
                addition.encode(encoder);
                encoder.slot(result);
            }
            Self::Special => {}
        }
    }
}

impl Encode for Recipe<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.string(self.kind);
        encoder.string(self.id);
        self.data.encode(encoder);
    }
}

impl Encode for UpdateRecipes<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.varint(self.recipes.len() as i32);
        for recipe in &self.recipes {
            recipe.encode(encoder);
        }
    }
}

/// Reads the body of the recipe whose type identifier is `kind`.
///
/// The identifier is namespaced (`minecraft:smelting`), so only the part after
/// the last `:` is matched. An unknown type is an error: its body has no length
/// prefix, so guessing would desynchronise the rest of the packet.
fn recipe_data<'a>(decoder: &mut Decoder<'a>, kind: &str) -> Result<RecipeData<'a>, Error> {
    let name = kind.rsplit(':').next().unwrap_or(kind);

    Ok(match name {
        "crafting_shapeless" => RecipeData::Shapeless {
            group: decoder.string()?,
            ingredients: counted(decoder, |d| Ingredient::decode(d))?,
            result: Slot::decode(decoder)?,
        },
        "crafting_shaped" => {
            let width = decoder.varint()?;
            let height = decoder.varint()?;
            let group = decoder.string()?;
            // The grid is `width * height` entries, not a counted array.
            let cells = width.get().max(0) as usize * height.get().max(0) as usize;
            if cells > pcore::decoder::MAX_ELEMENTS {
                return Err(Error::LenLimit(cells));
            }
            let mut ingredients = Vec::with_capacity(cells);
            for _ in 0..cells {
                ingredients.push(Ingredient::decode(decoder)?);
            }
            RecipeData::Shaped {
                width,
                height,
                group,
                ingredients,
                result: Slot::decode(decoder)?,
            }
        }
        "smelting" | "blasting" | "smoking" | "campfire_cooking" => RecipeData::Cooking {
            group: decoder.string()?,
            ingredient: Ingredient::decode(decoder)?,
            result: Slot::decode(decoder)?,
            experience: decoder.f32()?,
            cooking_time: decoder.varint()?,
        },
        "stonecutting" => RecipeData::Stonecutting {
            group: decoder.string()?,
            ingredient: Ingredient::decode(decoder)?,
            result: Slot::decode(decoder)?,
        },
        "smithing" => RecipeData::Smithing {
            base: Ingredient::decode(decoder)?,
            addition: Ingredient::decode(decoder)?,
            result: Slot::decode(decoder)?,
        },
        // Every `crafting_special_*` recipe has an empty body.
        _ if name.starts_with("crafting_special_") => RecipeData::Special,
        _ => return Err(Error::Nbt("unknown recipe type")),
    })
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

impl Encode for TagGroup<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.string(self.name);
        encoder.encode(&self.entries);
    }
}

impl Encode for TagRegistry<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.string(self.registry);
        encoder.varint(self.tags.len() as i32);
        for tag in &self.tags {
            tag.encode(encoder);
        }
    }
}

impl Encode for UpdateTags<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.varint(self.registries.len() as i32);
        for registry in &self.registries {
            registry.encode(encoder);
        }
    }
}
