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