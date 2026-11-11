use std::{env, mem::transmute, net::SocketAddr, sync::Arc, time::Duration};

use tokio::net::TcpListener;
use tracing::{error, info};

use crate::{
    connections::Connector, packets::{
        Packet, Side, State, UnsureThreshold, c2s::{self, packets::C2SPacket},
    }, scripts::{Engine, HandlerOptions, OwnType, Script},
};

mod buffer;
mod connections;
mod ids;
mod packets;
mod scripts;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    scripting_stuff().await;
    todo!("Stopped after doing the scripting stuff");

    dotenvy::dotenv().expect("Failed to initialize .env. Make sure you created the file");

    let bind_addr: SocketAddr = env::var("BIND_ADDR")
        .expect("Please, set BIND_ADDR in .env (example: BIND_ADDR=127.0.0.1:12955)")
        .parse()
        .expect("Invalid BIND_ADDR");

    let magic_number = env::var("MARK")
        .expect("Please, set MARK in .env (example: MARK=12332)")
        .parse::<u32>()
        .expect("Failed to parse the mark as u32");

    let connector = Arc::new(Connector::new(bind_addr, magic_number));
    let listener = TcpListener::bind(bind_addr)
        .await
        .unwrap_or_else(|e| panic!("Failed to start the listener on {bind_addr}: {e}"));

    info!("listening on {bind_addr}");

    loop {
        let (stream, addr) = match listener.accept().await {
            Ok(accepted) => accepted,
            Err(e) => {
                error!("accept failed: {e}");
                continue;
            }
        };

        let connector = connector.clone();
        tokio::spawn(async move {
            if let Err(e) = connector.connection(stream, addr).await {
                error!("[{addr}] connection error: {e}");
            }
        });
    }
}

async fn scripting_stuff() {
    let script =
        Script::parse(include_bytes!("../tests/scripts/test_script.tar").to_vec()).unwrap();
    let code = script.main_code().unwrap().to_owned();
    let engine = Engine::new(Arc::new(script)).unwrap();
    engine.set_globals().unwrap();
    tokio::spawn({
        let engine = engine.clone(); // Doesn't really cost anything since that's an Arc<...>
        async move {
            engine.run(&code).await.unwrap();
        }
    });
    let threshold = UnsureThreshold::new();
    threshold.set(256);
    tokio::time::sleep(Duration::from_millis(200)).await; // Wait till it loads
    let packet = Packet::C2S(C2SPacket::Status(c2s::status::Status::PingRequest(
        c2s::status::PingRequest { payload: 42 },
    )));
    let Some(own_type) = engine.own_type(0, packets::State::Handshake, packets::Side::C2S) else {
        panic!("Packet handler is not yet registered");
    };
    let packet = packet.to_owned();
    handle_packet(
        &engine,
        packets::State::Handshake,
        packets::Side::C2S,
        &packet,
        &threshold,
        0,
        own_type
    ).await;

    tokio::time::sleep(Duration::from_millis(400)).await; // And till it handles it
}

async fn handle_packet<'a>(engine: &Engine, state: State, side: Side, packet: &'a Packet<'static>, threshold: &'a UnsureThreshold, packet_id: u8, own_type: OwnType) {
    let func_data = engine
        .get_func(
            packet_id,
            state,
            side,
            own_type,
        )
        .unwrap()
        .unwrap();

    let packet_join_handler = tokio::spawn({
        let engine = engine.clone();
        let threshold = threshold.clone();
        let packet = packet.clone();

        async move {
            let result = engine.on_packet(
                func_data.function,
                OwnType::ReadInject,
                0,
                HandlerOptions::new(&packet, threshold, false),
            ).await;
            info!("Result with packet 0: {:?}", result.unwrap());
        }
    });
    
    if func_data.is_blocking {
        packet_join_handler.await.unwrap();
    };
}