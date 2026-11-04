//! The per-direction relay loop: read, parse, write.
//!
//! One task owns each direction, and that task is the only writer of its
//! socket half. That is the whole safety argument: TCP is a byte stream, so the
//! only way to corrupt it is to interleave two writers or to write a partial
//! frame. Both are impossible here — every write is a complete frame, and every
//! write happens in this task.
//!
//! Injected packets arrive on a channel and are written by the same task, so
//! they cannot interleave with real traffic either. They do not need to wait
//! for a real packet to arrive: a partial packet lives in `meta.stream_data`,
//! not in the socket, so writing an injection in between reads is safe.

use std::{borrow::Cow, net::SocketAddr, sync::Arc};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::mpsc::Sender,
};
use tracing::error;

use crate::{
    connections::{Channel, Injection, Stream, meta::Meta},
    packets::{Packet, Parsed, Parser, Side},
};

use super::testing_will_be_removed::on_packet;

/// Relays one direction of the connection until either side closes.
///
/// `stream` is the side being read from and `stream.tx` the side being written
/// to, so the same function serves both directions. Which one it is gets
/// decided from the local address: the listener binds `local_addr`, so a socket
/// whose peer is `local_addr` is the client->server direction.
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
            // `read` is cancel-safe: losing the race does not drop bytes.
            result = stream.rx.read(&mut buffer) => {
                let len = result?;
                if len == 0 {
                    stream.tx.shutdown().await?;
                    eyre::bail!(
                        "[{:?} -> {:?}] The stream is ended",
                        client_addr,
                        target_addr,
                    );
                }
                forward_real(&parser, side, &mut meta, &mut stream, &buffer[..len], &mut channel.tx).await?;
            }

            // `recv` is cancel-safe too. A closed channel is fatal: it lives as
            // long as the connection, so closing it is a bug in the caller.
            injection = channel.rx.recv() => {
                let injection = injection
                    .ok_or_else(|| eyre::eyre!("The injection channel is closed"))?;
                forward_injected(&parser, side, &mut meta, &mut stream, injection, &mut channel.tx).await?;
            }
        }
    }
}

/// Forwards every complete packet currently sitting in the receive buffer.
///
/// One `read` can carry several packets, so the buffer is drained completely.
/// A framing error is fatal: the next packet boundary cannot be found, so the
/// stream is unrecoverable. A body that fails to decode is not — the boundary
/// is known, so the frame is forwarded untouched and the loop continues.
async fn forward_real(
    parser: &Arc<Parser>,
    side: Side,
    meta: &mut Meta,
    stream: &mut Stream,
    bytes: &[u8],
    channel_tx: &mut Sender<Injection>,
) -> eyre::Result<()> {
    meta.stream_data.update(bytes)?;

    loop {
        let parsed = parser.try_parse(side, meta)?;
        if parsed.consumed().is_none() {
            // Only a partial packet is buffered; wait for the next read.
            return Ok(());
        }

        // `Parsed` carries the frame, so the receive buffer is not borrowed a
        // second time here. The borrow still has to end before `clear()`, hence
        // the inner block.
        {
            let frame = match parsed {
                Parsed::Packet { packet, frame, .. } => {
                    on_parsed_packet(packet, parser, channel_tx, frame).await
                }
                Parsed::BodyError { error, frame, .. } => {
                    log_body_error(side, &error, bytes);
                    Cow::Borrowed(frame)
                }
                Parsed::Incomplete => unreachable!("consumed() returned Some for Incomplete"),
            };
            stream.tx.write_all(&frame).await?;
        }

        meta.stream_data.clear();
    }
}

/// Writes every frame carried by one injection.
///
/// The injection holds complete frames, so nothing is re-framed here. A frame
/// that does not parse is a bug in the injector, not a reason to drop the
/// connection: it is logged and skipped.
async fn forward_injected(
    parser: &Arc<Parser>,
    side: Side,
    meta: &mut Meta,
    stream: &mut Stream,
    mut injection: Injection,
    channel_tx: &mut Sender<Injection>,
) -> eyre::Result<()> {
    while !injection.bytes.is_empty() {
        let parsed = parser.parse_injected(side, &injection.bytes, &mut meta.decompress_buffer)?;
        let Some(consumed) = parsed.consumed() else {
            error!(
                remaining = injection.bytes.len(),
                "injected bytes do not form a complete frame; dropping the rest"
            );
            return Ok(());
        };

        let frame = match parsed {
            Parsed::Packet { packet, frame, .. } if injection.changeable => {
                on_parsed_packet(packet, parser, channel_tx, frame).await
            }
            Parsed::Packet { frame, .. } => Cow::Borrowed(frame),
            Parsed::BodyError { error, frame, .. } => {
                log_body_error(side, &error, &injection.bytes);
                Cow::Borrowed(frame)
            }
            Parsed::Incomplete => unreachable!("consumed() returned Some for Incomplete"),
        };

        stream.tx.write_all(&frame).await?;
        injection.bytes.drain(..consumed);
    }
    Ok(())
}

fn log_body_error(side: Side, error: &eyre::Report, bytes: &[u8]) {
    error!(
        "[{:?}] parse error: {} got buffer: {:?} (could be not full, that's end part, max 50 bytes)",
        side,
        error,
        &bytes[..bytes.len().min(50)]
    );
}

/// Runs the rewrite hook and applies its result.
///
/// The injection is queued with `try_send`, not `send().await`: the channel
/// belongs to the *other* relay, and blocking here would let a slow peer stall
/// this direction too. A full queue means the injector is producing faster than
/// the peer consumes, so the packet is dropped and logged rather than turning
/// into backpressure on the real stream.
async fn on_parsed_packet<'a>(
    packet: Packet<'a>,
    parser: &Arc<Parser>,
    channel_tx: &mut Sender<Injection>,
    packet_bytes: &'a [u8],
) -> Cow<'a, [u8]> {
    let update = on_packet(&packet, parser.threshold());

    if let Some(inject) = update.inject
        && let Err(e) = channel_tx.try_send(inject)
    {
        error!("dropping an injected packet: {e}");
    }

    match update.change_with {
        Some(rewritten) => Cow::Owned(rewritten),
        None => Cow::Borrowed(packet_bytes),
    }
}
