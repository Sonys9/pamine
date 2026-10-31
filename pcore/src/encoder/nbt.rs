//! Named Binary Tag writer.
//!
//! The mirror of [`decoder::nbt`](crate::decoder::nbt). Two details matter:
//!
//! - In 1.19.2 a root tag still carries a name, written as `00 00` when empty.
//! - An empty list still needs an element type; `TAG_End` is the usual one.

use crate::decoder::NbtTag;

use super::string::write_string_u16;

impl crate::Encode for NbtTag<'_> {
    fn encode(&self, out: &mut Vec<u8>) {
        // A root tag carries an empty name in 1.19.2.
        encode_named(out, "", self);
    }
}

/// Writes one named NBT tag: type id, `u16` name length, name, payload.
fn encode_named(out: &mut Vec<u8>, name: &str, tag: &NbtTag<'_>) {
    out.push(tag_id(tag));
    if !matches!(tag, NbtTag::End) {
        write_string_u16(out, name);
    }
    encode_payload(out, tag);
}

/// The wire type id of a tag.
fn tag_id(tag: &NbtTag<'_>) -> u8 {
    match tag {
        NbtTag::End => 0,
        NbtTag::Byte(_) => 1,
        NbtTag::Short(_) => 2,
        NbtTag::Int(_) => 3,
        NbtTag::Long(_) => 4,
        NbtTag::Float(_) => 5,
        NbtTag::Double(_) => 6,
        NbtTag::ByteArray(_) => 7,
        NbtTag::String(_) => 8,
        NbtTag::List(_) => 9,
        NbtTag::Compound(_) => 10,
        NbtTag::IntArray(_) => 11,
        NbtTag::LongArray(_) => 12,
    }
}

/// Writes a tag's payload, without the type id and name.
fn encode_payload(out: &mut Vec<u8>, tag: &NbtTag<'_>) {
    match tag {
        NbtTag::End => {}
        NbtTag::Byte(v) => out.push(*v as u8),
        NbtTag::Short(v) => out.extend_from_slice(&v.to_be_bytes()),
        NbtTag::Int(v) => out.extend_from_slice(&v.to_be_bytes()),
        NbtTag::Long(v) => out.extend_from_slice(&v.to_be_bytes()),
        NbtTag::Float(v) => out.extend_from_slice(&v.to_be_bytes()),
        NbtTag::Double(v) => out.extend_from_slice(&v.to_be_bytes()),
        NbtTag::ByteArray(v) => {
            out.extend_from_slice(&(v.len() as i32).to_be_bytes());
            out.extend_from_slice(v);
        }
        NbtTag::String(v) => write_string_u16(out, v),
        NbtTag::List(items) => {
            // An empty list still needs an element type; TAG_End is the usual one.
            let elem = items.first().map(tag_id).unwrap_or(0);
            out.push(elem);
            out.extend_from_slice(&(items.len() as i32).to_be_bytes());
            for item in items {
                encode_payload(out, item);
            }
        }
        NbtTag::Compound(entries) => {
            for (name, value) in entries {
                encode_named(out, name, value);
            }
            out.push(0); // TAG_End terminates the compound
        }
        NbtTag::IntArray(values) => {
            out.extend_from_slice(&(values.len() as i32).to_be_bytes());
            for v in values {
                out.extend_from_slice(&v.to_be_bytes());
            }
        }
        NbtTag::LongArray(values) => {
            out.extend_from_slice(&(values.len() as i32).to_be_bytes());
            for v in values {
                out.extend_from_slice(&v.to_be_bytes());
            }
        }
    }
}
