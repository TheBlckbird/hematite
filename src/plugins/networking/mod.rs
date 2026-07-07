mod handler;
mod handshake;
pub mod listener;

use std::net::{IpAddr, Ipv4Addr};

use derive_more::Deref;
use flume::{Receiver, Sender};
use hematite_ecs::prelude::*;
use tracing::error;

use crate::protocol::packets::{AllCBPackets, AllSBPackets};

pub struct NetworkingPlugin {
    to_bevy_rx: Receiver<AllSBPackets>,
    to_networking_tx: Sender<AllCBPackets>,
}

impl NetworkingPlugin {
    pub fn new(to_bevy_rx: Receiver<AllSBPackets>, to_networking_tx: Sender<AllCBPackets>) -> Self {
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
pub struct ToBevyReceiver(Receiver<AllSBPackets>);

impl ToBevyReceiver {
    pub fn new(rx: Receiver<AllSBPackets>) -> Self {
        Self(rx)
    }
}

#[derive(Resource, Deref)]
pub struct ToNetworkingSender(Sender<AllCBPackets>);

impl ToNetworkingSender {
    pub fn new(tx: Sender<AllCBPackets>) -> Self {
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

        commands.run_system_cached_with(AllSBPackets::send_event, incoming_packet);
    }
}
