//! Upstream socket setup and original-destination recovery.

use std::{
    os::fd::{AsRawFd, FromRawFd},
    time::Duration,
};

use socket2::{Domain, SockAddr, Type};
use tokio::net::TcpStream;

/// `EINPROGRESS`: a non-blocking `connect` that is still in flight.
///
/// This is not a failure — the socket becomes writable once the handshake
/// completes, which is what [`get_stream`] waits for.
pub const UNAVAILABLE_ERR: i32 = 115;

/// Opens a socket to `addr` and waits until it is connected.
///
/// The socket is created by hand rather than with `TcpStream::connect` because
/// it needs `SO_MARK` set before connecting: the mark is what lets the netfilter
/// rules tell this outbound connection apart from the redirected one, so it
/// must be in place before the SYN leaves.
pub async fn get_stream(addr: &SockAddr, magic_number: u32) -> eyre::Result<TcpStream> {
    let domain = if addr.is_ipv6() {
        Domain::IPV6
    } else {
        Domain::IPV4
    };
    let socket = socket2::Socket::new(domain, Type::STREAM, None)?;
    socket.set_mark(magic_number)?;
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
    // `writable()` only says the connect attempt finished; it does not say it
    // succeeded, so the pending error has to be collected explicitly.
    if let Some(e) = target_stream.take_error()? {
        eyre::bail!("{}", e);
    };
    Ok(target_stream)
}

/// Recovers the original destination of a redirected connection.
///
/// The socket is borrowed from `stream`, not owned: `from_raw_fd` takes
/// ownership, so it is wrapped in `ManuallyDrop` to stop it closing a
/// descriptor that `stream` still uses. A plain `mem::forget` would leak the
/// descriptor on the `?` path below.
#[inline(always)]
pub fn stream_to_target_addr(stream: &TcpStream) -> eyre::Result<SockAddr> {
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
