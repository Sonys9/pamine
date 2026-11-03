use crate::packets::StreamData;

pub struct Meta {
    pub stream_data: StreamData,
    pub decompress_buffer: Vec<u8>,
}

impl Meta {
    pub fn new() -> Self {
        Self {
            stream_data: StreamData::default(),
            decompress_buffer: Vec::new(),
        }
    }
}
