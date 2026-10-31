use std::{env, net::SocketAddr, sync::Arc};

use tokio::net::TcpListener;
use tracing::{error, info};

use crate::connections::Connector;

mod buffer;
mod connections;
mod packets;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
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
