mod handler;
mod handshake;
pub mod listener;

use std::net::{IpAddr, Ipv4Addr};

use derive_more::Deref;
use flume::{Receiver, Sender};
use hematite_ecs::prelude::*;
use tracing::error;

use crate::protocol::packets::{EngineCBPackets, EngineSBPackets};

pub struct NetworkingPlugin {
    to_bevy_rx: Receiver<EngineSBPackets>,
    to_networking_tx: Sender<EngineCBPackets>,
}

impl NetworkingPlugin {
    pub fn new(
        to_bevy_rx: Receiver<EngineSBPackets>,
        to_networking_tx: Sender<EngineCBPackets>,
    ) -> Self {
        Self {
            to_bevy_rx,
            to_networking_tx,
        }
    }
}

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ToBevyReceiver::new(self.to_bevy_rx.clone()))
            .insert_resource(ToNetworkingSender::new(self.to_networking_tx.clone()))
            .add_systems(Startup, read_packets);
    }
}

// MARK: Constants

const PORT: u16 = 25565;
const ADDRESS: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));

// MARK: Resources

#[derive(Resource, Deref)]
pub struct ToBevyReceiver(Receiver<EngineSBPackets>);

impl ToBevyReceiver {
    pub fn new(rx: Receiver<EngineSBPackets>) -> Self {
        Self(rx)
    }
}

#[derive(Resource, Deref)]
pub struct ToNetworkingSender(Sender<EngineCBPackets>);

impl ToNetworkingSender {
    pub fn new(tx: Sender<EngineCBPackets>) -> Self {
        Self(tx)
    }
}

fn read_packets(to_bevy_receiver: Res<ToBevyReceiver>, mut commands: Commands) {
    loop {
        let incoming_packet = match to_bevy_receiver.recv() {
            Ok(packet) => packet,
            Err(error) => {
                error!("Error receiving packet: {error}");
                return;
            }
        };

        commands.run_system_cached_with(EngineSBPackets::send_event, incoming_packet);
    }
}
