//! Packets queued for injection, and the result of the rewrite hook.

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
    /// like real traffic, which lets a chain of scripts modify each other's
    /// output. `false` forwards it verbatim, which is what a script usually
    /// wants for its own output.
    ///
    /// A `true` injection that produces another `true` injection of the same
    /// packet would loop forever; the relay caps the chain at
    /// [`MAX_REWRITE_DEPTH`](crate::connections::relay::MAX_REWRITE_DEPTH).
    pub changeable: bool,
    /// A complete frame, ready to be written to the socket.
    pub bytes: Vec<u8>,
}

impl Injection {
    /// Builds an injection from a complete frame.
    ///
    /// `bytes` must be what [`frame_packet`](crate::packets::frame_packet)
    /// returns for the threshold in effect at injection time.
    pub fn new(bytes: Vec<u8>, changeable: bool) -> Self {
        Self { changeable, bytes }
    }
}

/// What the rewrite hook wants done with a packet.
#[derive(Default)]
pub struct Update {
    /// Replacement frames for the packet being inspected.
    ///
    /// `Some(bytes)` writes `bytes` instead of the original packet; `None`
    /// forwards the original untouched. `Some(Vec::new())` drops the packet.
    pub change_with: Option<Vec<u8>>,
    /// An extra packet to queue for the opposite direction.
    pub inject: Option<Injection>,
}

impl Update {
    /// Leaves the packet untouched and injects nothing.
    pub fn none() -> Self {
        Self::default()
    }
}
