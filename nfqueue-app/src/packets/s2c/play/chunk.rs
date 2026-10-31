//! Chunk data, light updates and chunk tracking.

use pcore::{
    Decode, Encode,
    decoder::{Decoder, NbtTag, VarInt},
    encoder::Encoder,
    errors::Error,
};

#[derive(Decode, Encode, Debug, Clone)]
pub struct UnloadChunk {
    pub chunk_x: i32,
    pub chunk_z: i32,
}

/// Chunk data plus light updates.
///
/// Hand-written `Decode`: the wire layout has three parts the derive macro
/// cannot express.
///
/// - `data` is a byte array whose length is the `Size` field, i.e. the VarInt
///   prefix *is* `Size` — there is no separate field to read.
/// - `block_entities` is a VarInt **count** followed by that many structures,
///   not a byte length.
/// - the light section is four BitSets followed by two length-prefixed arrays
///   of 2048-byte arrays.
#[derive(Debug, Clone)]
pub struct ChunkDataAndUpdateLight<'a> {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub heightmaps: NbtTag<'a>,
    pub data: &'a [u8],
    pub block_entities: Vec<BlockEntity<'a>>,
    pub trust_edges: bool,
    pub light: LightData<'a>,
}

impl<'a> Decode<'a> for ChunkDataAndUpdateLight<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let chunk_x = decoder.i32()?;
        let chunk_z = decoder.i32()?;
        let heightmaps = decoder.nbt()?;
        // `slice()` consumes the `Size` VarInt as its own length prefix.
        let data = decoder.slice()?;

        let count = decoder.varint()?.get();
        if count < 0 {
            return Err(Error::Nbt("negative block entity count"));
        }
        let mut block_entities = Vec::with_capacity(count as usize);
        for _ in 0..count {
            block_entities.push(BlockEntity::decode(decoder)?);
        }

        let trust_edges = decoder.bool()?;
        let light = LightData::decode(decoder)?;

        Ok(Self {
            chunk_x,
            chunk_z,
            heightmaps,
            data,
            block_entities,
            trust_edges,
            light,
        })
    }
}

impl Encode for ChunkDataAndUpdateLight<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.i32(self.chunk_x);
        encoder.i32(self.chunk_z);
        encoder.nbt(&self.heightmaps);
        // `bytes()` writes the `Size` VarInt prefix, which is the field itself.
        encoder.bytes(self.data);
        encoder.varint(self.block_entities.len() as i32);
        for entity in &self.block_entities {
            entity.encode(encoder);
        }
        encoder.bool(self.trust_edges);
        self.light.encode(encoder);
    }
}

/// One block entity inside a chunk.
#[derive(Debug, Clone)]
pub struct BlockEntity<'a> {
    /// `(blockX & 15) << 4 | (blockZ & 15)`
    pub packed_xz: u8,
    pub y: i16,
    pub kind: VarInt,
    pub data: NbtTag<'a>,
}

impl<'a> Decode<'a> for BlockEntity<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        Ok(Self {
            packed_xz: decoder.u8()?,
            y: decoder.i16()?,
            kind: decoder.varint()?,
            data: decoder.nbt()?,
        })
    }
}

impl Encode for BlockEntity<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.u8(self.packed_xz);
        encoder.i16(self.y);
        encoder.varint(self.kind.get());
        encoder.nbt(&self.data);
    }
}

/// Light section shared by `Chunk Data` and `Update Light`.
#[derive(Debug, Clone)]
pub struct LightData<'a> {
    pub sky_light_mask: &'a [u8],
    pub block_light_mask: &'a [u8],
    pub empty_sky_light_mask: &'a [u8],
    pub empty_block_light_mask: &'a [u8],
    pub sky_light: Vec<&'a [u8]>,
    pub block_light: Vec<&'a [u8]>,
}

impl<'a> Decode<'a> for LightData<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let sky_light_mask = bitset(decoder)?;
        let block_light_mask = bitset(decoder)?;
        let empty_sky_light_mask = bitset(decoder)?;
        let empty_block_light_mask = bitset(decoder)?;
        let sky_light = light_arrays(decoder)?;
        let block_light = light_arrays(decoder)?;
        Ok(Self {
            sky_light_mask,
            block_light_mask,
            empty_sky_light_mask,
            empty_block_light_mask,
            sky_light,
            block_light,
        })
    }
}

/// A BitSet is a VarInt count of `i64` words, then that many words.
fn bitset<'a>(decoder: &mut Decoder<'a>) -> Result<&'a [u8], Error> {
    let count = decoder.varint()?.get();
    if count < 0 {
        return Err(Error::Nbt("negative bitset length"));
    }
    decoder.read_len(count as usize * 8)
}

/// A length-prefixed array of length-prefixed 2048-byte arrays.
fn light_arrays<'a>(decoder: &mut Decoder<'a>) -> Result<Vec<&'a [u8]>, Error> {
    let count = decoder.varint()?.get();
    if count < 0 {
        return Err(Error::Nbt("negative light array count"));
    }
    let mut out = Vec::with_capacity(count as usize);
    for _ in 0..count {
        out.push(decoder.slice()?);
    }
    Ok(out)
}

impl Encode for LightData<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        write_bitset(encoder, self.sky_light_mask);
        write_bitset(encoder, self.block_light_mask);
        write_bitset(encoder, self.empty_sky_light_mask);
        write_bitset(encoder, self.empty_block_light_mask);
        write_light_arrays(encoder, &self.sky_light);
        write_light_arrays(encoder, &self.block_light);
    }
}

/// Writes a BitSet: a VarInt word count, then the words.
fn write_bitset(encoder: &mut Encoder, words: &[u8]) {
    encoder.varint((words.len() / 8) as i32);
    encoder.raw(words);
}

/// Writes a length-prefixed array of length-prefixed arrays.
fn write_light_arrays(encoder: &mut Encoder, arrays: &[&[u8]]) {
    encoder.varint(arrays.len() as i32);
    for array in arrays {
        encoder.bytes(array);
    }
}

/// Light-only update for a chunk. Shares the light section with
/// [`ChunkDataAndUpdateLight`], so it needs the same hand-written `Decode`.
#[derive(Debug, Clone)]
pub struct UpdateLight<'a> {
    pub chunk_x: VarInt,
    pub chunk_z: VarInt,
    pub trust_edges: bool,
    pub light: LightData<'a>,
}

impl<'a> Decode<'a> for UpdateLight<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        Ok(Self {
            chunk_x: decoder.varint()?,
            chunk_z: decoder.varint()?,
            trust_edges: decoder.bool()?,
            light: LightData::decode(decoder)?,
        })
    }
}

impl Encode for UpdateLight<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.varint(self.chunk_x.get());
        encoder.varint(self.chunk_z.get());
        encoder.bool(self.trust_edges);
        self.light.encode(encoder);
    }
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct SetCenterChunk {
    pub chunk_x: VarInt,
    pub chunk_z: VarInt,
}
