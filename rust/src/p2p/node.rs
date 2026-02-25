use std::net::SocketAddr;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::time::Duration;

use crate::network::protocol::{MessagePayload, ProtocolMessage, HANDSHAKE_TIMEOUT};
use super::framing::{read_frame, write_frame};
use super::transport::TcpTransport;

// After input EOF, wait this long for in-flight messages to arrive.
// Keeps tests deterministic on slow machines without adding hard sleeps.
const DRAIN_TIMEOUT_MS: u64 = 500;

#[derive(Debug, thiserror::Error)]
pub enum P2pError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Handshake timed out")]
    HandshakeTimeout,

    #[error("Message too large: {size} bytes")]
    MessageTooLarge { size: usize },

    #[error("Connection closed by peer")]
    ConnectionClosed,
}

/// A single active P2P connection between two peers.
///
/// Obtain via `P2pNode::listen` or `P2pNode::connect`.
/// Consumes itself in `run_chat_loop` / `run_chat_loop_with_input`.
pub struct P2pNode {
    local_name: String,
    local_id: String,
    peer_name: String,
    peer_id: String,
    reader: OwnedReadHalf,
    writer: OwnedWriteHalf,
}

impl P2pNode {
    /// Bind `port`, accept one incoming connection, perform handshake.
    pub async fn listen(port: u16, name: impl Into<String>) -> Result<Self, P2pError> {
        let listener = TcpTransport::listen(port).await?;
        let (stream, _) = listener.accept().await?;

        let local_name = name.into();
        let local_id = uuid::Uuid::new_v4().to_string();
        let (mut reader, mut writer) = stream.into_split();

        // Listener sends its handshake first (mirrors Briar's convention).
        let hs = ProtocolMessage::create_handshake(
            local_id.clone(),
            local_name.clone(),
            format!("0.0.0.0:{port}"),
            vec![],
        );
        write_frame(&mut writer, &hs).await?;

        let remote_hs = tokio::time::timeout(
            Duration::from_secs(HANDSHAKE_TIMEOUT),
            read_frame(&mut reader),
        )
        .await
        .map_err(|_| P2pError::HandshakeTimeout)??;

        let (peer_name, peer_id) = extract_peer_identity(&remote_hs)?;

        Ok(Self { local_name, local_id, peer_name, peer_id, reader, writer })
    }

    /// Connect to `addr`, perform handshake.
    pub async fn connect(addr: SocketAddr, name: impl Into<String>) -> Result<Self, P2pError> {
        let stream = TcpTransport::connect(addr).await?;

        let local_name = name.into();
        let local_id = uuid::Uuid::new_v4().to_string();
        let (mut reader, mut writer) = stream.into_split();

        // Connector receives the listener's handshake first.
        let remote_hs = tokio::time::timeout(
            Duration::from_secs(HANDSHAKE_TIMEOUT),
            read_frame(&mut reader),
        )
        .await
        .map_err(|_| P2pError::HandshakeTimeout)??;

        let (peer_name, peer_id) = extract_peer_identity(&remote_hs)?;

        let hs = ProtocolMessage::create_handshake(
            local_id.clone(),
            local_name.clone(),
            addr.to_string(),
            vec![],
        );
        write_frame(&mut writer, &hs).await?;

        Ok(Self { local_name, local_id, peer_name, peer_id, reader, writer })
    }

    /// Name the remote peer reported during handshake.
    pub fn peer_name(&self) -> &str {
        &self.peer_name
    }

    /// Interactive chat loop reading from `stdin`.
    ///
    /// Returns when the user sends EOF (Ctrl-D on Unix) or the peer disconnects.
    pub async fn run_chat_loop(self) -> Result<(), P2pError> {
        let stdin = BufReader::new(tokio::io::stdin());
        self.run_chat_loop_with_input(stdin).await.map(|_| ())
    }

    /// Chat loop with a pluggable input source — used by tests to inject
    /// scripted messages without touching real stdin.
    ///
    /// Returns all text messages received from the peer during the session.
    /// After input EOF the function waits up to `DRAIN_TIMEOUT_MS` for any
    /// in-flight messages before returning, which keeps test assertions reliable.
    pub async fn run_chat_loop_with_input<R: tokio::io::AsyncBufRead + Unpin>(
        mut self,
        mut input: R,
    ) -> Result<Vec<String>, P2pError> {
        let mut received: Vec<String> = Vec::new();
        let peer_name = self.peer_name.clone();

        println!(
            "[ShadowGhost] Connected to {}. Type messages and press Enter. EOF (Ctrl-D) to quit.",
            peer_name
        );

        loop {
            let mut line = String::new();
            tokio::select! {
                result = input.read_line(&mut line) => {
                    let n = result?;
                    if n == 0 {
                        // EOF from input: drain then exit
                        break;
                    }
                    let content = line.trim_end_matches(['\n', '\r']).to_string();
                    if content.is_empty() {
                        continue;
                    }
                    let msg = ProtocolMessage::create_text_message(
                        self.local_id.clone(),
                        self.peer_id.clone(),
                        content,
                        uuid::Uuid::new_v4().to_string(),
                    );
                    write_frame(&mut self.writer, &msg).await?;
                }

                result = read_frame(&mut self.reader) => {
                    match result {
                        Ok(msg) => {
                            if let Some(text) = msg.get_text_content() {
                                println!("[{}] {}", peer_name, text);
                                received.push(text);
                            }
                        }
                        Err(P2pError::ConnectionClosed) => {
                            println!("[ShadowGhost] {} disconnected.", peer_name);
                            return Ok(received);
                        }
                        Err(e) => return Err(e),
                    }
                }
            }
        }

        // Drain: collect any frames still in-flight after input EOF.
        loop {
            match tokio::time::timeout(
                Duration::from_millis(DRAIN_TIMEOUT_MS),
                read_frame(&mut self.reader),
            )
            .await
            {
                Ok(Ok(msg)) => {
                    if let Some(text) = msg.get_text_content() {
                        println!("[{}] {}", peer_name, text);
                        received.push(text);
                    }
                }
                // Timeout or any error ends the drain.
                _ => break,
            }
        }

        Ok(received)
    }
}

fn extract_peer_identity(msg: &ProtocolMessage) -> Result<(String, String), P2pError> {
    match msg.get_handshake_info() {
        Some(info) => Ok((info.peer_name.clone(), info.peer_id.clone())),
        None => {
            // Tolerate messages where the handshake payload is in the legacy
            // flat fields (sender_id / content) rather than the structured payload.
            if matches!(msg.payload, MessagePayload::Empty) && !msg.sender_id.is_empty() {
                Ok((msg.sender_id.clone(), msg.sender_id.clone()))
            } else {
                Err(P2pError::Protocol(
                    "expected Handshake payload in first message".to_string(),
                ))
            }
        }
    }
}
