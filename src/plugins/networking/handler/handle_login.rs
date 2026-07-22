use tokio::net::TcpStream;
use tracing::error;
use uuid::Uuid;

use crate::{
    plugins::networking::handler::into_raw_packet,
    protocol::{
        data_types::game_profile::GameProfile,
        packets::{
            NetworkingCBPackets, RoutedCBPacket,
            login::{clientbound::LoginSuccess, serverbound::LoginStart},
        },
    },
};

pub async fn handle_login_start(login_start: LoginStart, socket: &mut TcpStream) -> GameProfile {
    let uuid = Uuid::new_v3(
        &Uuid::NAMESPACE_OID,
        format!("OfflinePlayer:{}", login_start.name).as_bytes(),
    );

    let profile = GameProfile {
        uuid,
        username: login_start.name,
        properties: Box::default(),
    };

    let packet = into_raw_packet(RoutedCBPacket::Networking(
        NetworkingCBPackets::LoginSuccess(LoginSuccess {
            profile: profile.clone(),
        }),
    ));

    if let Err(err) = packet.send_via_socket(socket).await {
        error!("Error sending packet: {err:?}");
    }

    profile
}
