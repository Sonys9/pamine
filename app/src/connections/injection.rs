/// A packet queued for injection into one direction of the connection.
///
/// `bytes` is a complete frame — `len:VarInt [data_len:VarInt] body` — exactly
/// what [`frame_packet`](crate::packets::frame_packet) produces. The relay does
/// not re-frame it, so the injector is responsible for using the threshold that
/// is in effect at the time of injection.
pub struct Injection {
    /// Whether the rewrite hook may replace this packet.
    ///
    /// `true` runs the packet through
    /// [`on_packet`](crate::connections::testing_will_be_removed::on_packet)
    /// like real traffic; `false` forwards it verbatim, which is what an
    /// injector usually wants.
    pub changeable: bool,
    /// A complete frame, ready to be written to the socket.
    pub bytes: Vec<u8>,
}

impl Injection {
    /// Builds an injection from a complete frame.
    ///
    /// `bytes` must be what [`frame_packet`](crate::packets::frame_packet)
    /// returns for the threshold in effect at injection time.
    #[allow(dead_code)] // public API for injectors; nothing in the crate calls it yet
    pub fn new(bytes: Vec<u8>, changeable: bool) -> Self {
        Self { changeable, bytes }
    }
}

pub struct Update {
    pub change_with: Option<Vec<u8>>,
    pub inject: Option<Injection>,
}
