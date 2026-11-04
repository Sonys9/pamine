//! Connection handling: accept a redirected TCP stream, recover its original
//! destination and relay bytes in both directions.
//!
//! Module layout:
//!
//! - [`socket`] — opening the upstream socket and recovering the original
//!   destination from the netfilter redirect
//! - [`relay`] — the per-direction read/parse/write loop
//! - [`testing_will_be_removed`] — packet inspection and rewriting hooks
//!
//! [`Connector`] itself only wires the three together: it resolves the target,
//! opens the upstream socket and then races the two relay directions, so the
//! first one to finish (or fail) tears down the whole connection.

mod injection;
pub mod meta;
mod relay;
mod socket;
mod testing_will_be_removed;

use std::{net::SocketAddr, sync::Arc};

use tokio::{
    net::{
        TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
    sync::mpsc::{self, Receiver, Sender},
};
use tracing::info;

use crate::{
    connections::{injection::Injection, meta::Meta},
    packets::Parser,
};

use socket::{get_stream, stream_to_target_addr};

pub struct Connector {
    pub local_addr: SocketAddr,
    pub magic_number: u32,
}

impl Connector {
    #[inline]
    pub fn new(local_addr: SocketAddr, magic_number: u32) -> Self {
        Self {
            local_addr,
            magic_number,
        }
    }

    pub async fn connection(&self, stream: TcpStream, addr: SocketAddr) -> eyre::Result<()> {
        let target = stream_to_target_addr(&stream)?;
        info!("{:?} -> {:?}", addr, target.as_socket());

        let target_stream = get_stream(&target, self.magic_number).await?;

        // For the main stream
        let (stream_reader, stream_writer) = stream.into_split();
        let (target_stream_reader, target_stream_writer) = target_stream.into_split();

        // For sending custom packets
        let (source_tx, source_rx) = mpsc::channel::<Injection>(64);
        let (target_tx, target_rx) = mpsc::channel::<Injection>(64);

        // One parser is shared by both directions: the state machine and the
        // compression threshold are per-connection, not per-direction.
        let parser = Arc::new(Parser::new());
        tokio::select! {
            // Clone doesn't cost anything atp because it's an Arc
            res = relay::connection_side(
                Stream::new(stream_reader, target_stream_writer),
                self.local_addr,
                parser.clone(),
                Meta::new(),
                Channel::new(source_rx, target_tx)
            ) => res,
            res = relay::connection_side(
                Stream::new(target_stream_reader, stream_writer),
                self.local_addr,
                parser,
                Meta::new(),
                Channel::new(target_rx, source_tx)
            ) => res,
        }
    }
}

pub struct Channel {
    pub rx: Receiver<Injection>,
    pub tx: Sender<Injection>,
}

impl Channel {
    fn new(rx: Receiver<Injection>, tx: Sender<Injection>) -> Self {
        Self { rx, tx }
    }
}

pub struct Stream {
    pub rx: OwnedReadHalf,
    pub tx: OwnedWriteHalf,
}

impl Stream {
    fn new(rx: OwnedReadHalf, tx: OwnedWriteHalf) -> Self {
        Self { rx, tx }
    }
}
