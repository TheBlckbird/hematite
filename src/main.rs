use std::process::exit;

use tracing::{error, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

use crate::{
    app::app,
    plugins::networking::listener::run_server,
    protocol::packets::{EngineCBPackets, EngineSBPackets},
};

mod app;
mod core;
mod plugins;
mod protocol;

fn main() {
    // initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    // Create channels between tokio for networking and bevy
    let (to_bevy_tx, to_bevy_rx) = flume::unbounded::<EngineSBPackets>();
    let (to_networking_tx, to_networking_rx) = flume::unbounded::<EngineCBPackets>();

    // Spawn a thread for tokio and networking
    std::thread::spawn(|| {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(run_server(to_bevy_tx, to_networking_rx));
    });

    // Start the ECS
    let run_result = app(to_bevy_rx, to_networking_tx);

    if let Err(run_error) = run_result {
        error!("Error running application: {run_error}");
        exit(1);
    }
}
