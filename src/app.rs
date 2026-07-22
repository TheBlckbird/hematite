use flume::{Receiver, Sender};
use hematite_ecs::prelude::*;

use crate::{
    plugins::networking::NetworkingPlugin,
    protocol::packets::{EngineCBPackets, EngineSBPackets},
};

pub fn app(
    to_bevy_rx: Receiver<EngineSBPackets>,
    to_networking_tx: Sender<EngineCBPackets>,
) -> anyhow::Result<()> {
    App::new()
        .add_plugins(NetworkingPlugin::new(to_bevy_rx, to_networking_tx))
        .run()
}
