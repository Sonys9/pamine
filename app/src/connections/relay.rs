//! The per-direction relay loop: read, parse, write.

use std::{borrow::Cow, net::SocketAddr, sync::Arc};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
    sync::{Mutex, MutexGuard},
};
use tracing::error;

use crate::packets::{Parser, Side};

use super::testing_will_be_removed::on_packet;

/// Relays one direction of the connection until either side closes.
///
/// `stream` is the side being read from and `target_stream` the side being
/// written to, so the same function serves both directions. Which one it is
/// gets decided from the local address: the listener binds `local_addr`, so a
/// socket whose peer is `local_addr` is the client->server direction.
pub async fn connection_side(
    mut stream: OwnedReadHalf,
    mut target_stream: OwnedWriteHalf,
    local_addr: SocketAddr,
    parser: Arc<Mutex<Parser>>,
) -> eyre::Result<()> {
    let target_addr = target_stream.local_addr()?;
    let client_addr = stream.local_addr()?;
    let side = if target_addr == local_addr {
        Side::S2C
    } else {
        Side::C2S
    };

    let mut buffer = [0u8; 1024 * 4];
    loop {
        let len = stream.read(&mut buffer).await?;
        if len == 0 {
            target_stream.shutdown().await?;
            eyre::bail!(
                "[{:?} -> {:?}] The stream is ended",
                client_addr,
                target_addr,
            );
        };

        let mut parser = parser.lock().await;
        parser.update(side, &buffer[..len]);
        // One read can carry several packets, so drain the buffer completely.
        while let Some(packet) = parse_packet(&mut parser, side, &buffer[..len]) {
            target_stream.write_all(&packet).await?;
            parser.clear(side);
        }
        drop(parser);
    }
}

/// Parses at most one packet and returns the bytes to forward.
///
/// Returns `None` when the buffer holds only a partial packet, in which case
/// the caller should read more bytes and retry.
///
/// The returned bytes are always complete *frames* (`len:VarInt
/// [data_len:VarInt] body`), never bare bodies: the peer reads frames, so a
/// body without a length prefix desynchronises the stream. A rewrite may emit
/// several frames back to back — TCP is a byte stream, so the peer reads them
/// one at a time, and one `write_all` is cheaper than one syscall per packet.
fn parse_packet<'a>(
    parser: &'a mut MutexGuard<'_, Parser>,
    side: Side,
    buffer: &[u8],
) -> Option<Cow<'a, [u8]>> {
    // Read the threshold before `try_parse` borrows the parser: a hand-built
    // frame has to carry the same `data_len` header the peer expects.
    let threshold = parser.threshold();
    match parser.try_parse(side) {
        // `None` means the buffer holds only a partial packet;
        // wait for the next read instead of logging an error.
        Ok(None) => return None,
        Ok(Some(packet)) => {
            if let Some(framed) = on_packet(&packet, threshold) {
                return Some(Cow::Owned(framed));
            };
        }
        Err(e) => {
            error!(
                "[{:?}] parse error: {} got buffer: {:?} (could be not full, that's end part, max 50 bytes)",
                side,
                e,
                &buffer[..buffer.len().min(50)]
            );
        }
    };
    // Nothing was rewritten: forward the original bytes untouched.
    Some(Cow::Borrowed(parser.packet_bytes(side)))
}
