use std::{net::SocketAddr, process::exit};

use flume::{Receiver, Sender};
use tokio::net::TcpListener;
use tracing::{error, info};

use crate::{
    plugins::networking::{ADDRESS, PORT, handler::handle_client},
    protocol::packets::{AllCBPackets, AllSBPackets},
};

pub async fn run_server(
    to_bevy_tx: Sender<AllSBPackets>,
    to_networking_rx: Receiver<AllCBPackets>,
) {
    let address = SocketAddr::new(ADDRESS, PORT);
    let Ok(listener) = TcpListener::bind(address).await else {
        error!("An error occured trying to bind to {address}");
        exit(1);
    };

    info!("Started server on {address}");

    loop {
        let (socket, _) = match listener.accept().await {
            Ok(socket) => socket,
            Err(error) => {
                error!("An error occurred trying to connect a client: {error}");
                continue;
            }
        };

        let to_bevy_tx = to_bevy_tx.clone();
        let to_networking_rx = to_networking_rx.clone();
        tokio::spawn(async move { handle_client(socket, to_bevy_tx, to_networking_rx).await });
    }
}
