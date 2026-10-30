//! Command graph and tab-completion responses.

use pcore::{
    Decode,
    decoder::{Decoder, VarInt},
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

/// Reads the properties of the parser with the given id.
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
        31 => CommandParser::ScoreHolder {
            flags: decoder.u8()?,
        },
        43 => CommandParser::Time {
            min: decoder.i32()?,
        },
        44..=48 => CommandParser::Resource {
            registry: decoder.string()?,
        },
        other => return Err(Error::UnknownOpcode(other)),
    })
}
