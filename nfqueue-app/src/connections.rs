use std::{
    net::SocketAddr,
    os::fd::{AsRawFd, FromRawFd},
    sync::Arc,
    time::Duration,
};

use socket2::{Domain, SockAddr, Type};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
    sync::Mutex,
};
use tracing::{error, info};

use crate::packets::{Parser, Side, parser};

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

            // Scope the lock so it is released before the write below. Holding it
            // across `write_all` would let a slow peer block the other direction.
            {
                let mut parser = parser.lock().await;
                parser.update(side, &buffer[..len]);
                loop {
                    match parser.try_parse(side) {
                        Ok(packet) => {
                            info!("[{:?}] parsed packet: {:?}", side, packet);
                            parser.clear(side);
                        }
                        Err(e) => {
                            error!("[{:?}] {}", side, e);
                            break;
                        }
                    };
                    if parser.is_clear(side) {
                        break;
                    };
                }
            }

            target_stream.write_all(&buffer[..len]).await?;
        }
    }
}

#[inline]
fn stream_to_target_addr(stream: &TcpStream) -> eyre::Result<SockAddr> {
    let fd = stream.as_raw_fd();
    let socket = unsafe { socket2::Socket::from_raw_fd(fd) };
    let target = socket.original_dst_v4()?;
    std::mem::forget(socket);
    Ok(target)
}
