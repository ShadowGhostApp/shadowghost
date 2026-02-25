use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};

use super::node::P2pError;

// NOTE: future transports (QUIC, BLE) expose the same connect/listen interface.
// When adding a new transport, create a new struct here and wire it into P2pNode
// via a type parameter or a factory argument.
pub struct TcpTransport;

impl TcpTransport {
    pub async fn connect(addr: SocketAddr) -> Result<TcpStream, P2pError> {
        TcpStream::connect(addr).await.map_err(P2pError::Io)
    }

    pub async fn listen(port: u16) -> Result<TcpListener, P2pError> {
        TcpListener::bind(format!("0.0.0.0:{port}"))
            .await
            .map_err(P2pError::Io)
    }
}
