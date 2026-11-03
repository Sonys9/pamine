//! Command graph and tab-completion responses.

use pcore::{
    Decode, Encode,
    decoder::{Decoder, VarInt},
    encoder::Encoder,
    errors::Error,
};

use crate::packets::{counted, optional_chat};

/// One entry of a tab-completion response.
#[derive(Debug, Clone)]
pub struct SuggestionMatch<'a> {
    pub text: &'a str,
    /// The tooltip is a `bool`-gated chat component.
    pub tooltip: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct CommandSuggestionsResponse<'a> {
    pub id: VarInt,
    pub start: VarInt,
    pub length: VarInt,
    pub matches: Vec<SuggestionMatch<'a>>,
}

impl<'a> Decode<'a> for CommandSuggestionsResponse<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let id = decoder.varint()?;
        let start = decoder.varint()?;
        let length = decoder.varint()?;
        let matches = counted(decoder, |d| {
            Ok(SuggestionMatch {
                text: d.string()?,
                tooltip: optional_chat(d)?,
            })
        })?;
        Ok(Self {
            id,
            start,
            length,
            matches,
        })
    }
}

impl Encode for SuggestionMatch<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.string(self.text);
        match self.tooltip {
            Some(tooltip) => {
                encoder.bool(true);
                encoder.string(tooltip);
            }
            None => encoder.bool(false),
        }
    }
}

impl Encode for CommandSuggestionsResponse<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.varint(self.id.get());
        encoder.varint(self.start.get());
        encoder.varint(self.length.get());
        encoder.encode(&self.matches);
    }
}

/// The command graph. `nodes` is a VarInt **count** of variable-length node
/// structures, so it has to be parsed element by element.
#[derive(Debug, Clone)]
pub struct Commands<'a> {
    pub nodes: Vec<CommandNode<'a>>,
    pub root_index: VarInt,
}

/// Node type, taken from the low two bits of `flags`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandNodeKind {
    Root,
    Literal,
    Argument,
}

/// The parser-specific properties of an argument node.
///
/// Parsers without properties map to [`CommandParser::None`]; the ones that do
/// carry data are modelled explicitly so the cursor always lands on the next
/// node.
#[derive(Debug, Clone)]
pub enum CommandParser<'a> {
    /// No properties on the wire.
    None,
    /// `brigadier:double` — optional min/max.
    Double { min: Option<f64>, max: Option<f64> },
    /// `brigadier:float` — optional min/max.
    Float { min: Option<f32>, max: Option<f32> },
    /// `brigadier:integer` — optional min/max.
    Integer { min: Option<i32>, max: Option<i32> },
    /// `brigadier:long` — optional min/max.
    Long { min: Option<i64>, max: Option<i64> },
    /// `brigadier:string` — 0 = single word, 1 = quotable, 2 = greedy.
    String { behavior: VarInt },
    /// `minecraft:entity` — 0x01 single, 0x02 players only.
    Entity { flags: u8 },
    /// `minecraft:score_holder` — 0x01 allows multiple.
    ScoreHolder { flags: u8 },
    /// `minecraft:time` — minimum duration in ticks.
    Time { min: i32 },
    /// `minecraft:resource*` — the registry suggestions come from.
    Resource { registry: &'a str },
}

/// One node of the command graph.
///
/// `flags` bit 0x03 selects the node type, 0x04 marks an executable node,
/// 0x08 a redirect, 0x10 a suggestions type and 0x20 a restricted node.
#[derive(Debug, Clone)]
pub struct CommandNode<'a> {
    pub flags: u8,
    pub children: Vec<VarInt>,
    pub redirect: Option<VarInt>,
    pub name: Option<&'a str>,
    /// Parser id of an argument node.
    pub parser_id: Option<VarInt>,
    /// Parser properties of an argument node.
    pub parser: Option<CommandParser<'a>>,
    pub suggestions: Option<&'a str>,
}

impl CommandNode<'_> {
    /// Node type from the low two bits of `flags`.
    pub fn kind(&self) -> CommandNodeKind {
        match self.flags & 0x03 {
            0 => CommandNodeKind::Root,
            1 => CommandNodeKind::Literal,
            _ => CommandNodeKind::Argument,
        }
    }

    /// True when the node stack up to this point is a valid command.
    pub fn is_executable(&self) -> bool {
        self.flags & 0x04 != 0
    }

    /// True when the node requires a permission level above 0.
    pub fn is_restricted(&self) -> bool {
        self.flags & 0x20 != 0
    }
}

impl<'a> Decode<'a> for Commands<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let count = decoder.varint()?.get();
        if count < 0 {
            return Err(Error::Nbt("negative command node count"));
        }
        let mut nodes = Vec::with_capacity(count as usize);
        for _ in 0..count {
            nodes.push(CommandNode::decode(decoder)?);
        }
        let root_index = decoder.varint()?;
        Ok(Self { nodes, root_index })
    }
}

impl<'a> Decode<'a> for CommandNode<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        const TYPE_MASK: u8 = 0x03;
        const REDIRECT: u8 = 0x08;
        const SUGGESTIONS: u8 = 0x10;

        let flags = decoder.u8()?;

        let child_count = decoder.varint()?.get();
        if child_count < 0 {
            return Err(Error::Nbt("negative child count"));
        }
        let mut children = Vec::with_capacity(child_count as usize);
        for _ in 0..child_count {
            children.push(decoder.varint()?);
        }

        let redirect = if flags & REDIRECT != 0 {
            Some(decoder.varint()?)
        } else {
            None
        };

        // Root nodes (type 0) carry no name. Literal (1) and argument (2) nodes
        // both start with a name; argument nodes add a parser id and then the
        // parser-specific properties.
        let (name, parser_id, parser) = match flags & TYPE_MASK {
            0 => (None, None, None),
            1 => (Some(decoder.string()?), None, None),
            _ => {
                let name = decoder.string()?;
                let parser_id = decoder.varint()?;
                let parser = parser_properties(decoder, parser_id.get())?;
                (Some(name), Some(parser_id), Some(parser))
            }
        };

        let suggestions = if flags & SUGGESTIONS != 0 {
            Some(decoder.string()?)
        } else {
            None
        };

        Ok(Self {
            flags,
            children,
            redirect,
            name,
            parser_id,
            parser,
            suggestions,
        })
    }
}

impl Encode for Commands<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.varint(self.nodes.len() as i32);
        for node in &self.nodes {
            node.encode(encoder);
        }
        encoder.varint(self.root_index.get());
    }
}

impl Encode for CommandNode<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.u8(self.flags);
        encoder.varint(self.children.len() as i32);
        for child in &self.children {
            encoder.varint(child.get());
        }
        if let Some(redirect) = self.redirect {
            encoder.varint(redirect.get());
        }
        if let Some(name) = self.name {
            encoder.string(name);
        }
        if let Some(parser_id) = self.parser_id {
            encoder.varint(parser_id.get());
            if let Some(parser) = &self.parser {
                parser.encode(encoder);
            }
        }
        if let Some(suggestions) = self.suggestions {
            encoder.string(suggestions);
        }
    }
}

impl Encode for CommandParser<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        match self {
            Self::None => {}
            Self::Double { min, max } => {
                let mut flags = 0u8;
                if min.is_some() {
                    flags |= 0x01;
                }
                if max.is_some() {
                    flags |= 0x02;
                }
                encoder.u8(flags);
                if let Some(min) = min {
                    encoder.f64(*min);
                }
                if let Some(max) = max {
                    encoder.f64(*max);
                }
            }
            Self::Float { min, max } => {
                let mut flags = 0u8;
                if min.is_some() {
                    flags |= 0x01;
                }
                if max.is_some() {
                    flags |= 0x02;
                }
                encoder.u8(flags);
                if let Some(min) = min {
                    encoder.f32(*min);
                }
                if let Some(max) = max {
                    encoder.f32(*max);
                }
            }
            Self::Integer { min, max } => {
                let mut flags = 0u8;
                if min.is_some() {
                    flags |= 0x01;
                }
                if max.is_some() {
                    flags |= 0x02;
                }
                encoder.u8(flags);
                if let Some(min) = min {
                    encoder.i32(*min);
                }
                if let Some(max) = max {
                    encoder.i32(*max);
                }
            }
            Self::Long { min, max } => {
                let mut flags = 0u8;
                if min.is_some() {
                    flags |= 0x01;
                }
                if max.is_some() {
                    flags |= 0x02;
                }
                encoder.u8(flags);
                if let Some(min) = min {
                    encoder.i64(*min);
                }
                if let Some(max) = max {
                    encoder.i64(*max);
                }
            }
            Self::String { behavior } => encoder.varint(behavior.get()),
            Self::Entity { flags } => encoder.u8(*flags),
            Self::ScoreHolder { flags } => encoder.u8(*flags),
            Self::Time { min } => encoder.i32(*min),
            Self::Resource { registry } => encoder.string(registry),
        }
    }
}

/// Reads the properties of the parser with the given id.
///
/// The ids are the `minecraft:command_argument_type` registry, which is dense
/// `0..=47` in 1.19.2. Getting one wrong desynchronises the whole packet: the
/// cursor lands mid-field and the next node is read from garbage, which shows
/// up as `Invalid UTF-8` far away from the actual mistake.
///
/// An unknown id is an error: the wiki is explicit that the rest of the packet
/// cannot be unpacked once an unknown parser is met.
fn parser_properties<'a>(
    decoder: &mut Decoder<'a>,
    parser_id: i32,
) -> Result<CommandParser<'a>, Error> {
    Ok(match parser_id {
        // brigadier:bool, and every parser without properties.
        0 => CommandParser::None,
        1 => {
            let flags = decoder.u8()?;
            CommandParser::Float {
                min: if flags & 0x01 != 0 {
                    Some(decoder.f32()?)
                } else {
                    None
                },
                max: if flags & 0x02 != 0 {
                    Some(decoder.f32()?)
                } else {
                    None
                },
            }
        }
        2 => {
            let flags = decoder.u8()?;
            CommandParser::Double {
                min: if flags & 0x01 != 0 {
                    Some(decoder.f64()?)
                } else {
                    None
                },
                max: if flags & 0x02 != 0 {
                    Some(decoder.f64()?)
                } else {
                    None
                },
            }
        }
        3 => {
            let flags = decoder.u8()?;
            CommandParser::Integer {
                min: if flags & 0x01 != 0 {
                    Some(decoder.i32()?)
                } else {
                    None
                },
                max: if flags & 0x02 != 0 {
                    Some(decoder.i32()?)
                } else {
                    None
                },
            }
        }
        4 => {
            let flags = decoder.u8()?;
            CommandParser::Long {
                min: if flags & 0x01 != 0 {
                    Some(decoder.i64()?)
                } else {
                    None
                },
                max: if flags & 0x02 != 0 {
                    Some(decoder.i64()?)
                } else {
                    None
                },
            }
        }
        5 => CommandParser::String {
            behavior: decoder.varint()?,
        },
        6 => CommandParser::Entity {
            flags: decoder.u8()?,
        },
        // `minecraft:score_holder` — 0x01 allows multiple.
        29 => CommandParser::ScoreHolder {
            flags: decoder.u8()?,
        },
        // `minecraft:time` — minimum duration in ticks.
        42 => CommandParser::Time {
            min: decoder.i32()?,
        },
        // `minecraft:resource_or_tag` and `minecraft:resource` both carry the
        // registry identifier they suggest from.
        43..=44 => CommandParser::Resource {
            registry: decoder.string()?,
        },
        // Every other 1.19.2 parser (7-28, 30-41, 45-47) has no properties.
        7..=47 => CommandParser::None,
        other => return Err(Error::UnknownOpcode(other)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The ids are the `minecraft:command_argument_type` registry. A wrong id
    /// here does not fail locally: it shifts the cursor and the *next* node is
    /// read from garbage, which surfaces as `Invalid UTF-8` somewhere else in
    /// the packet. So the ids are pinned explicitly.
    #[test]
    fn parser_ids_match_the_1_19_2_registry() {
        // (id, name, bytes of properties on the wire)
        let cases: &[(i32, &str, &[u8])] = &[
            (0, "brigadier:bool", &[]),
            (1, "brigadier:float", &[0x00]),
            (2, "brigadier:double", &[0x00]),
            (3, "brigadier:integer", &[0x00]),
            (4, "brigadier:long", &[0x00]),
            (5, "brigadier:string", &[0x00]),
            (6, "minecraft:entity", &[0x00]),
            (7, "minecraft:game_profile", &[]),
            (8, "minecraft:block_pos", &[]),
            (9, "minecraft:column_pos", &[]),
            (10, "minecraft:vec3", &[]),
            (11, "minecraft:vec2", &[]),
            (12, "minecraft:block_state", &[]),
            (13, "minecraft:block_predicate", &[]),
            (14, "minecraft:item_stack", &[]),
            (15, "minecraft:item_predicate", &[]),
            (16, "minecraft:color", &[]),
            (17, "minecraft:component", &[]),
            (18, "minecraft:message", &[]),
            (19, "minecraft:nbt_compound_tag", &[]),
            (20, "minecraft:nbt_tag", &[]),
            (21, "minecraft:nbt_path", &[]),
            (22, "minecraft:objective", &[]),
            (23, "minecraft:objective_criteria", &[]),
            (24, "minecraft:operation", &[]),
            (25, "minecraft:particle", &[]),
            (26, "minecraft:angle", &[]),
            (27, "minecraft:rotation", &[]),
            (28, "minecraft:scoreboard_slot", &[]),
            (29, "minecraft:score_holder", &[0x00]),
            (30, "minecraft:swizzle", &[]),
            (31, "minecraft:team", &[]),
            (32, "minecraft:item_slot", &[]),
            (33, "minecraft:resource_location", &[]),
            (34, "minecraft:mob_effect", &[]),
            (35, "minecraft:function", &[]),
            (36, "minecraft:entity_anchor", &[]),
            (37, "minecraft:int_range", &[]),
            (38, "minecraft:float_range", &[]),
            (39, "minecraft:item_enchantment", &[]),
            (40, "minecraft:entity_summon", &[]),
            (41, "minecraft:dimension", &[]),
            (42, "minecraft:time", &[0x00, 0x00, 0x00, 0x00]),
            (43, "minecraft:resource_or_tag", &[0x00]),
            (44, "minecraft:resource", &[0x00]),
            (45, "minecraft:template_mirror", &[]),
            (46, "minecraft:template_rotation", &[]),
            (47, "minecraft:uuid", &[]),
        ];

        for (id, name, properties) in cases {
            let mut decoder = Decoder::new(properties);
            let parsed = parser_properties(&mut decoder, *id)
                .unwrap_or_else(|e| panic!("{name} (id {id}) failed to parse: {e}"));
            assert_eq!(
                decoder.remaining(),
                0,
                "{name} (id {id}) left {} bytes unread, parsed as {parsed:?}",
                decoder.remaining()
            );
        }
    }

    /// The two ids that were wrong before: `score_holder` and `time` were
    /// treated as property-less, and `resource_or_tag` was read as an `i32`.
    #[test]
    fn score_holder_and_time_read_their_properties() {
        let mut decoder = Decoder::new(&[0x01]);
        let parsed = parser_properties(&mut decoder, 29).unwrap();
        assert!(matches!(parsed, CommandParser::ScoreHolder { flags: 1 }));
        assert_eq!(decoder.remaining(), 0);

        let mut decoder = Decoder::new(&[0x00, 0x00, 0x00, 0x05]);
        let parsed = parser_properties(&mut decoder, 42).unwrap();
        assert!(matches!(parsed, CommandParser::Time { min: 5 }));
        assert_eq!(decoder.remaining(), 0);
    }

    /// `resource_or_tag` (43) carries a registry string, not an `i32`.
    #[test]
    fn resource_or_tag_reads_a_registry_string() {
        // VarInt length 4 followed by "item".
        let mut decoder = Decoder::new(&[0x04, b'i', b't', b'e', b'm']);
        let parsed = parser_properties(&mut decoder, 43).unwrap();
        match parsed {
            CommandParser::Resource { registry } => assert_eq!(registry, "item"),
            other => panic!("expected Resource, got {other:?}"),
        }
        assert_eq!(decoder.remaining(), 0);
    }

    /// `template_mirror` (45) has no properties, so nothing may be consumed.
    #[test]
    fn template_mirror_has_no_properties() {
        let mut decoder = Decoder::new(&[]);
        let parsed = parser_properties(&mut decoder, 45).unwrap();
        assert!(matches!(parsed, CommandParser::None));
        assert_eq!(decoder.remaining(), 0);
    }
}
