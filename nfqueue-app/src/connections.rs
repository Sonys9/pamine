use std::{
    borrow::Cow, net::SocketAddr, os::fd::{AsRawFd, FromRawFd}, sync::Arc, time::Duration,
};

use pcore::{Encode, encode};
use socket2::{Domain, SockAddr, Type};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt}, net::{
        TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    }, sync::{Mutex, MutexGuard},
};
use tracing::{error, info};

use crate::packets::{
    Packet::C2S,
    Parser, Side,
    c2s::{self, packets::C2SPacket, play::ChatMessage},
};

pub const UNAVAILABLE_ERR: i32 = 115;

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

        let target_stream = self.get_stream(&target).await?;
        let (stream_reader, stream_writer) = stream.into_split();
        let (target_stream_reader, target_stream_writer) = target_stream.into_split();

        let parser = Arc::new(Mutex::new(Parser::new()));
        tokio::select! {
            res = Self::connection_side(stream_reader, target_stream_writer, self.local_addr, parser.clone()) => res,
            res = Self::connection_side(target_stream_reader, stream_writer, self.local_addr, parser) => res,
        }
    }

    async fn get_stream(&self, addr: &SockAddr) -> eyre::Result<TcpStream> {
        let domain = if addr.is_ipv6() {
            Domain::IPV6
        } else {
            Domain::IPV4
        };
        let socket = socket2::Socket::new(domain, Type::STREAM, None)?;
        socket.set_mark(self.magic_number)?;
        socket.set_nonblocking(true)?;
        if let Err(e) = socket.connect(addr)
            && e.raw_os_error() != Some(UNAVAILABLE_ERR)
        {
            eyre::bail!("Failed to init a connection: {e}");
        };
        let target_stream = TcpStream::from_std(socket.into())?;
        tokio::select! {
            res = target_stream.writable() => res,
            _ = tokio::time::sleep(Duration::from_secs(10)) => eyre::bail!("Timeout"),
        }?;
        if let Some(e) = target_stream.take_error()? {
            eyre::bail!("{}", e);
        };
        Ok(target_stream)
    }

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
            loop {
                match parse_packet(&mut parser, side, &buffer[..len]) {
                    Some(packet) => target_stream.write_all(&packet).await?,
                    None => break
                };
                parser.clear(side);
            };
            drop(parser);
        }
    }
}

// Moved this code to the fn to decrease the nesting
#[inline(always)]
fn parse_packet<'a>(parser: &'a mut MutexGuard<'_, Parser>, side: Side, buffer: &[u8]) -> Option<Cow<'a, [u8]>> {
    match parser.try_parse(side) {
        // `None` means the buffer holds only a partial packet;
        // wait for the next read instead of logging an error.
        Ok(None) => return None,
        Ok(Some(packet)) => {
            if 
                let C2S(C2SPacket::Play(c2s::play::Play::ChatMessage(message))) = packet
                && let Some(new_packet) = on_message(message)
            {
                return Some(Cow::Owned(new_packet));
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
    Some(Cow::Borrowed(parser.packet_bytes(side)))
}

// Same as "parse_packet"
#[inline(always)]
fn on_message(mut message: ChatMessage<'_>) -> Option<Vec<u8>> {
    info!("The local player just typed '{}'", message.message);
    if message.message == "Hello, world!" {
        info!("Spoofing with the custom message");
        message.message = "!dlrow ,olleH";
        return Some(encode(&message))
    };
    None
}

/// Recovers the original destination of a redirected connection.
///
/// The socket is borrowed from `stream`, not owned: `from_raw_fd` takes
/// ownership, so it is wrapped in `ManuallyDrop` to stop it closing a
/// descriptor that `stream` still uses. A plain `mem::forget` would leak the
/// descriptor on the `?` path below.
#[inline(always)]
fn stream_to_target_addr(stream: &TcpStream) -> eyre::Result<SockAddr> {
    let fd = stream.as_raw_fd();
    let socket = std::mem::ManuallyDrop::new(unsafe { socket2::Socket::from_raw_fd(fd) });

    // `original_dst_v4` and `original_dst_v6` are separate socket options, so
    // the family has to be picked from the local address.
    let local = stream.local_addr()?;
    if local.is_ipv6() {
        Ok(socket.original_dst_v6()?)
    } else {
        Ok(socket.original_dst_v4()?)
    }
}
