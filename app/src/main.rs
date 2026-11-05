use std::{env, net::SocketAddr, sync::Arc, time::Duration};

use tokio::net::TcpListener;
use tracing::{error, info};

use crate::{connections::Connector, scripts::Engine};

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
    let engine = Engine::new().unwrap();
    engine.set_globals().unwrap();
    tokio::spawn({
        let engine = engine.clone(); // Doesn't really cost anything because that's an Arc<...>
        async move {
            engine.run(include_str!("scripts/test_script.lua")).await.unwrap();
        }
    });
    tokio::time::sleep(Duration::from_millis(200)).await; // Wait till it loads
    info!("Sending the packet");
    engine.on_packet(0, String::from("Foo")).await.unwrap();
    info!("Done");
    tokio::time::sleep(Duration::from_secs(2)).await; // And till it handles it
}