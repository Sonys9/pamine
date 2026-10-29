use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to decode")]
    Decode,

    #[error("Failed to read due to payload < type len: tried to read {0} bytes")]
    Read(usize),

    #[error("Read error")]
    ReadType,

    #[error("Invalid VarLong")]
    VarLong,

    #[error("Invalid VarInt")]
    VarInt,

    #[error("Unknown opcode: {0}")]
    UnknownOpcode(i32),
}