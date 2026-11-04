//! Applying the rewrite hook to a parsed packet.
//!
//! The relay parses frames; this module decides what to do with them. Keeping
//! the two apart means `relay.rs` reads as "read, parse, write" and the rewrite
//! rules — including the depth cap that stops a chain from looping — live in one
//! place.

use std::borrow::Cow;

use tokio::sync::mpsc::Sender;
use tracing::error;

use crate::{
    connections::Injection,
    packets::{Packet, Parser},
};

use super::testing_will_be_removed::on_packet;

/// Runs the rewrite hook on one packet and returns the bytes to write.
///
/// Three outcomes:
///
/// - the hook returns nothing → the original frame is forwarded untouched;
/// - the hook returns a non-`changeable` replacement → its bytes are written;
/// - the hook returns a `changeable` replacement → it is queued back into this
///   direction so the hook sees it again, and nothing is written now.
///
/// The last case is what lets a chain of scripts modify each other's output. It
/// is also the only way to loop, so the depth is capped at
/// [`MAX_REWRITE_DEPTH`]: once a packet has been rewritten that many times it is
/// written as-is instead of being queued again.
pub async fn apply<'a>(
    packet: Packet<'a>,
    parser: &Parser,
    own_tx: &Sender<Injection>,
    neighbour_tx: &Sender<Injection>,
    original: &'a [u8],
    is_injected: bool,
) -> Cow<'a, [u8]> {
    let update = on_packet(&packet, parser.threshold(), is_injected);

    if let Some(inject) = update.inject
        && let Err(e) = neighbour_tx.try_send(inject)
    {
        error!("dropping an injected packet: {e}");
    }

    let Some(mut change_with) = update.change_with else {
        return Cow::Borrowed(original);
    };

    if !change_with.changeable {
        return Cow::Owned(change_with.bytes);
    }

    if change_with.is_exhausted() {
        error!(
            depth = change_with.depth,
            "rewrite chain hit MAX_REWRITE_DEPTH, forwarding as-is"
        );
        return Cow::Owned(change_with.bytes);
    }

    change_with.bump_depth();
    if let Err(e) = own_tx.try_send(change_with) {
        error!("dropping a change_with packet: {e}");
    }

    // The replacement is queued, so nothing is written for this packet now.
    Cow::Borrowed(&[])
}
