//! 128-bit UUID writer.

use crate::decoder::Uuid;

impl crate::Encode for Uuid {
    fn encode(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.bytes);
    }
}
