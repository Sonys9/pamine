//! Packets queued for injection, and the result of the rewrite hook.

use pcore::{Encode, encode};

use crate::packets::{UnsureThreshold, frame_packet};

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
    pub fn new(bytes: Option<Vec<u8>>, changeable: bool) -> Self {
        Self { changeable, bytes: bytes.unwrap_or(Vec::new()) }
    }
    
    /// A shortcut for Injection::new(bytes, true);
    pub fn new_changeable(bytes: Option<Vec<u8>>) -> Self {
        Self::new(bytes, true)
    }

    /// A shortcut for Injection::new(bytes, false);
    pub fn new_unchangeable(bytes: Option<Vec<u8>>) -> Self {
        Self::new(bytes, false)
    }

    pub fn new_from_packet_changeable<T: Encode + ?Sized>(packet: &T, threshold: &UnsureThreshold) -> eyre::Result<Self> {
        let body = encode(packet);
        let framed = frame_packet(&body, threshold)?;
        Ok(Self::new_changeable(Some(framed)))
    }

    pub fn new_from_packet_unchangeable<T: Encode + ?Sized>(packet: &T, threshold: &UnsureThreshold) -> eyre::Result<Self> {
        let mut injection = Self::new_from_packet_changeable(packet, threshold)?;
        injection.changeable = false;
        Ok(injection)
    }
    
    pub fn update_from_packet<T: Encode + ?Sized>(&mut self, packet: &T, threshold: &UnsureThreshold) -> eyre::Result<()> {
        let body = encode(packet);
        let framed = frame_packet(&body, threshold)?;
        self.bytes.extend_from_slice(&framed);
        Ok(())
    }
}

/// What the rewrite hook wants done with a packet.
#[derive(Default)]
pub struct Update {
    /// Replacement frames for the packet being inspected.
    ///
    /// `Some(bytes)` writes `bytes` instead of the original packet; `None`
    /// forwards the original untouched. `Some(Vec::new())` drops the packet.
    pub change_with: Option<Injection>,
    /// An extra packet to queue for the opposite direction.
    pub inject: Option<Injection>,
}

impl Update {
    /// Leaves the packet untouched and injects nothing.
    pub fn none() -> Self {
        Self::default()
    }

    // To decrease the boilerplate
    pub fn new_with_change(change_with: Injection) -> Self {
        Self {
            change_with: Some(change_with),
            inject: None
        }
    }

    pub fn new_with_inject(inject: Injection) -> Self {
        Self {
            change_with: None,
            inject: Some(inject)
        }
    }

    pub fn new(change_with: Injection, inject: Injection) -> Self {
        Self {
            change_with: Some(change_with),
            inject: Some(inject)
        }
    }

    pub fn update_change(&mut self, new_change_with: Injection) {
        match self.change_with.as_mut() {
            Some(change_with) => change_with.bytes.extend_from_slice(&new_change_with.bytes),
            None => self.change_with = Some(new_change_with)
        }
    }

    pub fn update_inject(&mut self, new_inject: Injection) {
        match self.inject.as_mut() {
            Some(inject) => inject.bytes.extend_from_slice(&new_inject.bytes),
            None => self.inject = Some(new_inject)
        }
    }
}
