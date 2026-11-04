//! The per-direction relay loop: read, parse, write.

use std::{borrow::Cow, net::SocketAddr, sync::Arc};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::error;

use crate::{
    connections::{Channel, Stream, meta::Meta},
    packets::{Parser, Side},
};

use super::testing_will_be_removed::on_packet;

/// Relays one direction of the connection until either side closes.
///
/// `stream` is the side being read from and `target_stream` the side being
/// written to, so the same function serves both directions. Which one it is
/// gets decided from the local address: the listener binds `local_addr`, so a
/// socket whose peer is `local_addr` is the client->server direction.
pub async fn connection_side(
    mut stream: Stream,
    local_addr: SocketAddr,
    parser: Arc<Parser>,
    mut meta: Meta,
    mut channel: Channel,
) -> eyre::Result<()> {
    let target_addr = stream.tx.local_addr()?;
    let client_addr = stream.rx.local_addr()?;
    let side = if target_addr == local_addr {
        Side::S2C
    } else {
        Side::C2S
    };

    let mut buffer = [0u8; 1024 * 4];
    loop {
        tokio::select! {
            result = stream.rx.read(&mut buffer) =>
                on_real_packet(
                    &parser,
                    side,
                    (client_addr, target_addr),
                    result,
                    &mut meta,
                    &mut stream,
                    &buffer
                ).await?,
            packet = channel.rx.recv() => {
                while let Some(packet) = parse_packet(&parser, side, &[], &mut meta, &packet)? {
                    stream.tx.write_all(&packet).await?;
                }
            }
        };

        /*
        while let Some(packet) = parse_packet(&parser, side, &buffer[..len], &mut meta, )? {

        }*/
    }
}

#[inline(always)]
async fn on_real_packet(
    parser: &Arc<Parser>,
    side: Side,
    (client_addr, target_addr): (SocketAddr, SocketAddr),
    result: Result<usize, std::io::Error>,
    meta: &mut Meta,
    stream: &mut Stream,
    buffer: &[u8],
) -> eyre::Result<()> {
    let len = result?;
    if len == 0 {
        stream.tx.shutdown().await?;
        eyre::bail!(
            "[{:?} -> {:?}] The stream is ended",
            client_addr,
            target_addr,
        );
    };

    meta.stream_data.update(&buffer[..len])?;
    // One read can carry several packets, so drain the buffer completely.
    //
    // A parse error is fatal: the stream is desynchronised and there is no
    // way to find the next packet boundary, so the connection is torn down
    // instead of spinning on the same bytes forever.
    while let Some(packet) = parse_packet(parser, side, &buffer[..len], meta, &None)? {
        stream.tx.write_all(&packet).await?;
        meta.stream_data.clear();
    }
    Ok(())
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
///
/// # Errors
///
/// Only *framing* errors are fatal. A body that fails to decode is not: the
/// frame boundary is already known, so the packet can be forwarded untouched
/// and the stream stays in sync. This matters because the decoder is not
/// complete — an unknown or mis-modelled packet body must not kill a working
/// connection.
///
/// The two cases are told apart by the read cursor: [`Parser::try_parse`]
/// advances it as soon as the frame is validated, *before* dispatching the
/// body. So a cursor that moved means the boundary is known and the error came
/// from the body; a cursor that did not move means the frame itself is broken
/// and the next boundary cannot be found.
#[inline(always)]
fn parse_packet<'a>(
    parser: &'a Arc<Parser>,
    side: Side,
    buffer: &[u8], // used only for error logging
    meta: &'a mut Meta,
    custom: &Option<Vec<u8>>,
) -> eyre::Result<Option<Cow<'a, [u8]>>> {
    // Read the threshold before `try_parse` borrows the parser: a hand-built
    // frame has to carry the same `data_len` header the peer expects.
    let threshold = parser.threshold();
    let cursor_before = meta.stream_data.diff;

    match parser.try_parse(side, meta, custom) {
        // `None` means the buffer holds only a partial packet;
        // wait for the next read instead of logging an error.
        Ok(None) => return Ok(None),
        Ok(Some(packet)) => {
            if let Some(framed) = on_packet(&packet, threshold) {
                return Ok(Some(Cow::Owned(framed)));
            };
        }
        Err(e) => {
            error!(
                "[{:?}] parse error: {} got buffer: {:?} (could be not full, that's end part, max 50 bytes)",
                side,
                e,
                &buffer[..buffer.len().min(50)]
            );
            // The frame was validated and the cursor moved, so the packet
            // boundary is known: forward the packet as-is and keep going.
            // Only a frame-level failure leaves the cursor in place, and that
            // one is unrecoverable.
            if meta.stream_data.diff == cursor_before {
                return Err(e);
            }
        }
    };
    // Nothing was rewritten: forward the original bytes untouched.
    Ok(Some(Cow::Borrowed(meta.stream_data.packet_bytes())))
}
