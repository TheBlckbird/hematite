use tokio::{io::AsyncWriteExt, net::TcpStream};
use tracing::{error, info};

pub async fn disconnect_client_err(socket: TcpStream, error: impl std::error::Error) {
    error!("{error}");
    disconnect_client(socket).await;
}

pub async fn disconnect_client_msg(socket: TcpStream, error_message: impl ToString) {
    let error_message = error_message.to_string();
    error!("{error_message}");
    disconnect_client(socket).await;
}

pub async fn disconnect_client(mut socket: TcpStream) {
    info!("Disconnecting client...");
    // [TODO] send disconnect packet to client

    let shutdown_result = socket.shutdown().await;
    if let Err(error) = shutdown_result {
        error!("Error disconnecting client: {error}");
    }
}
