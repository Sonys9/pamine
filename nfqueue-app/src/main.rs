#![feature(ip_as_octets)]

use std::{
    env, io::Write, mem::transmute, net::{IpAddr, Ipv4Addr, SocketAddr}, slice::{from_raw_parts, from_raw_parts_mut}, sync::Arc, time::UNIX_EPOCH,
};

use etherparse::{
    IpHeaders, IpNumber, IpPayloadSlice, Ipv4Extensions, Ipv4Header, NetHeaders, NetSlice,
    PacketBuilder, PacketHeaders, SlicedPacket, TcpHeader, TcpSlice, TransportSlice::Tcp,
};
use nfq::{Message, Queue, Verdict};
use pcore::{
    Decode,
    decoder::{self, Decoder, VarInt},
};
use sudo::check;
use tokio::net::TcpListener;
use tracing::{error, info};

use crate::{
    buffer::BigBuffer, connections::Connector, packets::{Side, c2s, parser},
};

mod buffer;
mod connections;
mod packets;

pub const IP_HEADER_LEN: usize = 20;
const SPOOF_WITH: &'static [u8] = b"123456789012345678901";
const LOCAL_HOST: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 25565);

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().expect("Failed to initialize .env. Make sure you created the file");
    let bind_addr = env::var("BIND_ADDR")
        .expect("Please, set BIND_ADDR in .env (example: BIND_ADDR=127.0.0.1:12955")
        .parse()
        .expect("Invalid BIND_ADDR");
    let magic_number = env::var("MARK")
        .expect("Please, set MARK in .env (example: MARK=12332")
        .parse::<u32>()
        .expect("Failed to parse the mark as u32");

    let connector = Arc::new(Connector::new(bind_addr, magic_number));
    let listener = TcpListener::bind(&bind_addr)
        .await
        .expect(&format!("Failed to start the listener on {}", bind_addr));
    loop {
        let Ok((stream, addr)) = listener.accept().await else {
            continue;
        };
        let connector = connector.clone();
        tokio::spawn(async move {
            let Err(e) = connector.connection(stream, addr).await else {
                return;
            };
            error!("Unexpected socket error: {}", e);
        });
    }

    /*
    loop {

        let payload = raw_packet.get_payload();
        let Ok(packet) = parse(&payload) else {
           //  queue.verdict(raw_packet).unwrap();
            continue;
        };
        if packet.destination != LOCAL_HOST && packet.source != LOCAL_HOST {
            // queue.verdict(raw_packet).unwrap();
            continue;
        };
        if packet.tcp.syn() && !packet.tcp.ack() && !packet.tcp.fin() && !packet.tcp.rst() && packet.tcp.acknowledgment_number() == 0 {
            parser.reset();
            // queue.verdict(raw_packet).unwrap();
            continue;
        };
        if packet.content.is_empty() {
            // queue.verdict(raw_packet).unwrap();
            continue;
        };

        let side = if raw_packet.get_outdev() == 0 { Side::S2C } else { Side::C2S };
        parser.update(side, packet.content, packet.tcp.sequence_number());

        let mut session = parser.session(side);
        let packet = match session.try_parse() {
            Ok(packet) => packet,
            Err(e) => {
                error!("[{:?}] Got error: {}", side, e);
                // queue.verdict(raw_packet).unwrap();
                continue;
            },
        };
        info!("data: {:?}", packet);
        // queue.verdict(raw_packet).unwrap();
    }
    */
}

fn modify_packet<'a>(
    raw_packet: &'a Message,
    mut packet: Packet<'a>,
    buffer: &mut BigBuffer,
) -> eyre::Result<()> {
    let out_interface = raw_packet.get_outdev();
    let content = String::from_utf8_lossy(packet.content);
    if !content.contains("HTTP/1.1") && !content.contains("46.32.86.108") {
        eyre::bail!("Incorrect packet");
    }
    info!(
        "[{} from {} {} {} -> {}] {}",
        raw_packet.get_hw_protocol(),
        out_interface,
        if out_interface == 0 { "IN" } else { "OUT" },
        packet.source,
        packet.destination,
        content
    );
    if out_interface != 0 || !content.contains("46.32.86.108") {
        eyre::bail!("Incorrect packet");
    };
    /*let mut content_length_value_len: Option<usize> = None;
    let mut content_length_value_pos: Option<usize> = None;
    for line in packet.content.split(|&bytes| bytes == b'\n') {
        if line == b"\r" {
            break;
        };
        let mut values = line.split(|&bytes| bytes == b':');
        let (Some(name), Some(value)) = (values.next(), values.next()) else {
            eyre::bail!("Incorrect http header");
        };
        if name == b"Content-Length" {
            content_length_value_len = Some()
        };
    };*/
    if let Some(data_index) = packet
        .content
        .windows(4)
        .position(|bytes| bytes == b"\r\n\r\n")
    {
        packet.content = &mut packet.content[..data_index + 4];
    };
    let ttl = raw_packet.get_payload()[8];
    let dont_fragment = raw_packet.get_payload()[6] & 0b01000000 == 0b01000000;
    let tcp = packet.tcp.to_header();
    let headers_len = IP_HEADER_LEN + tcp.header_len();
    for _ in 0..headers_len {
        // not optimized but easy
        buffer.update(&[0], 1);
    }
    buffer.update(packet.content, packet.content.len());
    let start_pos = buffer.update(SPOOF_WITH, SPOOF_WITH.len());
    // content_buffer[..content_len].copy_from_slice(packet.content);
    // content_buffer[content_len..content_len + SPOOF_WITH.len()].copy_from_slice(SPOOF_WITH);
    // content_len += SPOOF_WITH.len();
    let total_len = headers_len + packet.content.len() + SPOOF_WITH.len();
    let builder = PacketBuilder::ip(IpHeaders::Ipv4(
        Ipv4Header {
            source: packet.source.ip().as_octets().try_into().unwrap(),
            destination: packet.destination.ip().as_octets().try_into().unwrap(),
            time_to_live: ttl,
            total_len: total_len as u16,
            protocol: IpNumber::TCP,
            identification: packet.id.unwrap(),
            dont_fragment,
            ..Default::default()
        },
        Ipv4Extensions::default(),
    ))
    .tcp_header(tcp);

    let write_start_index = start_pos - packet.content.len() - headers_len;
    let payload_start = start_pos - packet.content.len();
    let mut write_start: &mut [u8] = unsafe {
        let ptr = buffer.buffer.as_mut_ptr().add(write_start_index);
        from_raw_parts_mut(ptr, total_len)
    };
    let payload =
        &buffer.buffer[payload_start..payload_start + packet.content.len() + SPOOF_WITH.len()];
    info!(
        "from: {:?} ({}), payload: {:?} (len is {})",
        write_start,
        write_start_index,
        payload,
        packet.content.len() + SPOOF_WITH.len()
    );
    builder.write(&mut write_start, payload)?;
    info!(
        "Result: {:?} (stringify: {:?})",
        &buffer.buffer[write_start_index..buffer.len],
        String::from_utf8_lossy(&buffer.buffer[write_start_index..buffer.len])
    );
    Ok(())
}

#[allow(mutable_transmutes)]
fn parse<'a>(packet: &'a [u8]) -> eyre::Result<Packet<'a>> {
    let packet = SlicedPacket::from_ip(packet)?;
    let Tcp(tcp) = packet
        .transport
        .clone()
        .ok_or(eyre::Report::msg("No transport type found"))?
    else {
        eyre::bail!("Wrong transport protocol");
    };
    let (source_port, destination_port) = (tcp.source_port(), tcp.destination_port());
    let (id, source_ip, destination_ip) = match packet.net.as_ref() {
        Some(NetSlice::Ipv4(v4)) => {
            let header = v4.header();
            (
                Some(header.identification()),
                IpAddr::V4(header.source_addr()),
                IpAddr::V4(header.destination_addr()),
            )
        }
        _ => eyre::bail!("Wrong net type"),
    };
    let content = unsafe { transmute(tcp.payload()) };
    Ok(Packet {
        content,
        source: SocketAddr::new(source_ip, source_port),
        destination: SocketAddr::new(destination_ip, destination_port),
        tcp,
        id,
    })
}

#[derive(Debug)]
pub struct Packet<'a> {
    pub source: SocketAddr,
    pub destination: SocketAddr,
    pub content: &'a mut [u8],
    pub tcp: TcpSlice<'a>,
    pub id: Option<u16>,
}
