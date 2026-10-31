//! 128-bit UUID writer.

use crate::decoder::Uuid;

impl crate::Encode for Uuid {
    fn encode(&self, encoder: &mut crate::encoder::Encoder) {
        encoder.uuid(self);
    }
}
