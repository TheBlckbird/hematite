use hematite_ecs::prelude::*;
use tracing::error;

use crate::{
    plugins::networking::{NetworkingPlugin, listener::run_server},
    protocol::packets::{AllCBPackets, AllSBPackets},
};

mod plugins;
mod protocol;

fn main() {
    tracing_subscriber::fmt::init();

    let (to_bevy_tx, to_bevy_rx) = flume::unbounded::<AllSBPackets>();
    let (to_networking_tx, to_networking_rx) = flume::unbounded::<AllCBPackets>();

    std::thread::spawn(|| {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(run_server(to_bevy_tx, to_networking_rx));
    });

    let run_result = App::new()
        .add_plugins(NetworkingPlugin::new(to_bevy_rx, to_networking_tx))
        .run();

    if let Err(run_error) = run_result {
        error!("Error running application: {run_error}");
    }
}
