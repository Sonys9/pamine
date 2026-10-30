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

/// One node of the command graph.
///
/// `flags` bit 0x03 selects the node type: 0 = root, 1 = literal, 2 = argument.
/// Bit 0x04 marks an executable node, 0x08 a redirect and 0x10 suggestions.
#[derive(Debug, Clone)]
pub struct CommandNode<'a> {
    pub flags: u8,
    pub children: Vec<VarInt>,
    pub redirect: Option<VarInt>,
    pub name: Option<&'a str>,
    pub parser_id: Option<VarInt>,
    pub suggestions: Option<&'a str>,
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
        // both start with a name; argument nodes add a parser id and then
        // parser-specific properties, which have no length prefix and are
        // therefore not consumed here.
        let (name, parser_id) = match flags & TYPE_MASK {
            0 => (None, None),
            1 => (Some(decoder.string()?), None),
            _ => (Some(decoder.string()?), Some(decoder.varint()?)),
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
            suggestions,
        })
    }
}
