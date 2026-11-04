//! Packets queued for injection, and the result of the rewrite hook.

use pcore::{Encode, encode};

use crate::packets::{UnsureThreshold, frame_packet};

/// How many times one packet may pass through the rewrite hook.
///
/// A `changeable` injection is fed back into the hook, which lets a chain of
/// scripts modify each other's output. Without a cap, a script that rewrites a
/// packet into another `changeable` packet of the same kind would loop forever:
/// the relay would keep re-queueing it and never drain the channel.
pub const MAX_REWRITE_DEPTH: u8 = 64;

/// A packet queued for injection into one direction of the connection.
///
/// `bytes` is a complete frame — `len:VarInt [data_len:VarInt] body` — exactly
/// what [`frame_packet`](crate::packets::frame_packet) produces. The relay does
/// not re-frame it, so the injector is responsible for using the threshold that
/// is in effect at the time of injection.
#[derive(Debug, Clone)]
pub struct Injection {
    /// Whether the rewrite hook may replace this packet.
    ///
    /// `true` runs the packet through
    /// [`on_packet`](crate::connections::testing_will_be_removed::on_packet)
    /// like real traffic, which lets a chain of scripts modify each other's
    /// output. `false` forwards it verbatim, which is what a script usually
    /// wants for its own output.
    pub changeable: bool,
    /// A complete frame, ready to be written to the socket.
    pub bytes: Vec<u8>,
    /// How many times this packet has already been through the hook.
    ///
    /// Starts at `0` for a fresh injection and is bumped by the relay each time
    /// the packet is re-queued. Capped at [`MAX_REWRITE_DEPTH`].
    pub depth: u8,
}

impl Injection {
    /// An empty injection, used as a builder before appending packets.
    pub fn empty(changeable: bool) -> Self {
        Self {
            changeable,
            bytes: Vec::new(),
            depth: 0,
        }
    }

    /// An injection holding one already-framed packet.
    ///
    /// Useful when the caller already has the frame, e.g. when re-queueing a
    /// packet that came out of the parser.
    #[allow(dead_code)] // public API for injectors; nothing in the crate calls it yet
    pub fn from_frame(bytes: Vec<u8>, changeable: bool) -> Self {
        Self {
            changeable,
            bytes,
            depth: 0,
        }
    }

    /// Encodes and frames `packet` into a new injection.
    pub fn from_packet<T: Encode + ?Sized>(
        packet: &T,
        threshold: &UnsureThreshold,
        changeable: bool,
    ) -> eyre::Result<Self> {
        let mut injection = Self::empty(changeable);
        injection.push_packet(packet, threshold)?;
        Ok(injection)
    }

    /// Appends one more framed packet to this injection.
    ///
    /// Several frames concatenated are safe: TCP is a byte stream, so the peer
    /// reads them back one at a time.
    pub fn push_packet<T: Encode + ?Sized>(
        &mut self,
        packet: &T,
        threshold: &UnsureThreshold,
    ) -> eyre::Result<()> {
        let body = encode(packet);
        let framed = frame_packet(&body, threshold)?;
        self.bytes.extend_from_slice(&framed);
        Ok(())
    }

    /// True when there is nothing left to write.
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// True when this packet has been rewritten too many times.
    pub fn is_exhausted(&self) -> bool {
        self.depth >= MAX_REWRITE_DEPTH
    }

    /// Marks one more pass through the hook.
    pub fn bump_depth(&mut self) {
        self.depth = self.depth.saturating_add(1);
    }
}

/// What the rewrite hook wants done with a packet.
#[derive(Default, Debug)]
pub struct Update {
    /// Replacement for the packet being inspected.
    ///
    /// `Some(injection)` writes the injection's frames instead of the original
    /// packet; `None` forwards the original untouched. An injection with empty
    /// `bytes` drops the packet.
    pub change_with: Option<Injection>,
    /// An extra packet to queue for the opposite direction.
    pub inject: Option<Injection>,
}

impl Update {
    /// Leaves the packet untouched and injects nothing.
    pub fn none() -> Self {
        Self::default()
    }

    /// Replaces the packet, injects nothing.
    pub fn change(change_with: Injection) -> Self {
        Self {
            change_with: Some(change_with),
            inject: None,
        }
    }

    /// Forwards the packet, injects one extra.
    #[allow(dead_code)] // public API for scripts; nothing in the crate calls it yet
    pub fn inject(inject: Injection) -> Self {
        Self {
            change_with: None,
            inject: Some(inject),
        }
    }

    /// Replaces the packet and injects one extra.
    pub fn both(change_with: Injection, inject: Injection) -> Self {
        Self {
            change_with: Some(change_with),
            inject: Some(inject),
        }
    }

    /// Appends frames to the replacement, creating it if absent.
    ///
    /// The `changeable` flag of the existing injection wins: appending must not
    /// silently flip a packet from "forward verbatim" to "run through the hook
    /// again", because that is how a rewrite chain starts looping.
    #[allow(dead_code)] // public API for scripts; nothing in the crate calls it yet
    pub fn append_change(&mut self, extra: Injection) {
        match self.change_with.as_mut() {
            Some(existing) => existing.bytes.extend_from_slice(&extra.bytes),
            None => self.change_with = Some(extra),
        }
    }

    /// Appends frames to the injection, creating it if absent.
    ///
    /// Same flag rule as [`Self::append_change`].
    #[allow(dead_code)] // public API for scripts; nothing in the crate calls it yet
    pub fn append_inject(&mut self, extra: Injection) {
        match self.inject.as_mut() {
            Some(existing) => existing.bytes.extend_from_slice(&extra.bytes),
            None => self.inject = Some(extra),
        }
    }
}
