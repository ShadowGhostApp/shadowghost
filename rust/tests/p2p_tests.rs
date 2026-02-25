use shadowghost::network::protocol::ProtocolMessage;
use shadowghost::p2p::framing::{read_frame, write_frame};
use shadowghost::p2p::{P2pError, P2pNode};
use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::time::Duration;

// ── helpers ────────────────────────────────────────────────────────────────

/// Binds port 0, reads the OS-assigned port, then drops the listener.
/// There is a small race window before the caller uses the port, which is
/// acceptable in an isolated test environment.
async fn free_port() -> u16 {
    tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn sample_text_msg(sender: &str, recipient: &str, content: &str, id: &str) -> ProtocolMessage {
    ProtocolMessage::create_text_message(
        sender.to_string(),
        recipient.to_string(),
        content.to_string(),
        id.to_string(),
    )
}

// ── framing unit tests ─────────────────────────────────────────────────────

/// write_frame → read_frame over an in-memory duplex channel.
#[tokio::test]
async fn test_framing_roundtrip() {
    let (mut client, mut server) = tokio::io::duplex(8192);
    let original = sample_text_msg("alice", "bob", "hello", "id-1");

    write_frame(&mut client, &original).await.unwrap();

    let received = read_frame(&mut server).await.unwrap();
    assert_eq!(received.message_id, original.message_id);
    assert_eq!(received.get_text_content(), Some("hello".to_string()));
}

/// Ten messages sent sequentially arrive in the same order.
#[tokio::test]
async fn test_framing_multiple() {
    let (mut writer_stream, mut reader_stream) = tokio::io::duplex(64 * 1024);

    let write_task = tokio::spawn(async move {
        for i in 0..10u32 {
            let msg = sample_text_msg("a", "b", &format!("content_{i}"), &format!("id_{i}"));
            write_frame(&mut writer_stream, &msg).await.unwrap();
        }
    });

    let read_task = tokio::spawn(async move {
        let mut ids = Vec::new();
        for _ in 0..10 {
            let msg = read_frame(&mut reader_stream).await.unwrap();
            ids.push(msg.message_id.clone());
        }
        ids
    });

    write_task.await.unwrap();
    let ids = read_task.await.unwrap();

    assert_eq!(ids.len(), 10);
    for i in 0..10usize {
        assert_eq!(ids[i], format!("id_{i}"));
    }
}

/// A length prefix of u32::MAX must be rejected with MessageTooLarge before
/// any payload bytes are read, so the reader never tries to allocate 4 GB.
#[tokio::test]
async fn test_framing_oversized() {
    // Write only the 4-byte length header with MAX value — no payload.
    let data: Vec<u8> = u32::MAX.to_be_bytes().to_vec();
    let mut reader = std::io::Cursor::new(data);

    let result = read_frame(&mut reader).await;
    assert!(
        matches!(result, Err(P2pError::MessageTooLarge { .. })),
        "expected MessageTooLarge, got {:?}",
        result
    );
}

// ── integration tests ──────────────────────────────────────────────────────

/// Two nodes complete a handshake and both learn the other's name.
#[tokio::test]
async fn test_handshake() {
    let port = free_port().await;

    let listener_task = tokio::spawn(async move {
        let node = P2pNode::listen(port, "alice").await.unwrap();
        node.peer_name().to_string()
    });

    // Give the listener a moment to reach accept().
    tokio::time::sleep(Duration::from_millis(50)).await;

    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    let connector = P2pNode::connect(addr, "bob").await.unwrap();

    let listener_peer_name = tokio::time::timeout(Duration::from_secs(5), listener_task)
        .await
        .expect("listener timed out")
        .unwrap();

    assert_eq!(listener_peer_name, "bob");
    assert_eq!(connector.peer_name(), "alice");
}

/// Alice sends one message; Bob receives it verbatim.
#[tokio::test]
async fn test_send_receive() {
    let port = free_port().await;

    let listener_task = tokio::spawn(async move {
        let node = P2pNode::listen(port, "alice").await.unwrap();
        // alice sends one message, collects nothing received
        node.run_chat_loop_with_input(&b"hello bob\n"[..])
            .await
            .unwrap()
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    let bob_node = P2pNode::connect(addr, "bob").await.unwrap();
    // bob sends nothing, collects received messages
    let bob_received = bob_node
        .run_chat_loop_with_input(&b""[..])
        .await
        .unwrap();

    let alice_received = tokio::time::timeout(Duration::from_secs(5), listener_task)
        .await
        .expect("alice timed out")
        .unwrap();

    assert!(alice_received.is_empty());
    assert_eq!(bob_received, vec!["hello bob"]);
}

/// Both sides send messages and both receive the other's messages.
#[tokio::test]
async fn test_bidirectional() {
    let port = free_port().await;

    let listener_task = tokio::spawn(async move {
        let node = P2pNode::listen(port, "alice").await.unwrap();
        node.run_chat_loop_with_input(&b"from_alice\n"[..])
            .await
            .unwrap()
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    let bob_node = P2pNode::connect(addr, "bob").await.unwrap();
    let bob_received = bob_node
        .run_chat_loop_with_input(&b"from_bob\n"[..])
        .await
        .unwrap();

    let alice_received = tokio::time::timeout(Duration::from_secs(5), listener_task)
        .await
        .expect("alice timed out")
        .unwrap();

    assert!(
        alice_received.contains(&"from_bob".to_string()),
        "alice should have received bob's message; got: {alice_received:?}"
    );
    assert!(
        bob_received.contains(&"from_alice".to_string()),
        "bob should have received alice's message; got: {bob_received:?}"
    );
}

/// Connecting to a port that has no listener returns an IO error.
#[tokio::test]
async fn test_connection_refused() {
    // Port 1 is reserved and never open in user-space.
    let addr: SocketAddr = "127.0.0.1:1".parse().unwrap();
    let result = P2pNode::connect(addr, "x").await;
    assert!(
        matches!(result, Err(P2pError::Io(_))),
        "expected Io error, got {:?}",
        result
    );
}

/// When the connector drops its end, the listener exits cleanly without panic.
#[tokio::test]
async fn test_graceful_disconnect() {
    let port = free_port().await;

    let listener_task = tokio::spawn(async move {
        let node = P2pNode::listen(port, "alice").await.unwrap();
        // alice waits for messages; connector will disconnect first
        node.run_chat_loop_with_input(&b""[..]).await.unwrap()
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    // Connect, do handshake, then immediately drop — this closes the TCP stream.
    let _bob = P2pNode::connect(addr, "bob").await.unwrap();
    drop(_bob);

    // Listener should finish without panic within a reasonable timeout.
    tokio::time::timeout(Duration::from_secs(5), listener_task)
        .await
        .expect("listener hung after peer disconnect")
        .unwrap();
}

/// 100 sequential messages arrive at the receiver in the correct order.
#[tokio::test]
async fn test_100_messages_ordered() {
    let port = free_port().await;
    const N: usize = 100;

    let listener_task = tokio::spawn(async move {
        let node = P2pNode::listen(port, "sender").await.unwrap();
        let input: String = (0..N).map(|i| format!("msg_{i}\n")).collect();
        node.run_chat_loop_with_input(input.as_bytes())
            .await
            .unwrap()
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    let receiver = P2pNode::connect(addr, "receiver").await.unwrap();
    let received = receiver
        .run_chat_loop_with_input(&b""[..])
        .await
        .unwrap();

    tokio::time::timeout(Duration::from_secs(10), listener_task)
        .await
        .expect("sender timed out")
        .unwrap();

    assert_eq!(received.len(), N, "expected {N} messages, got {}", received.len());
    for i in 0..N {
        assert_eq!(received[i], format!("msg_{i}"), "wrong message at index {i}");
    }
}

// ── unit tests ─────────────────────────────────────────────────────────────

/// ProtocolMessage::create_handshake produces a structurally valid message.
#[test]
fn test_handshake_message_valid() {
    let msg = ProtocolMessage::create_handshake(
        "peer-id".to_string(),
        "Alice".to_string(),
        "127.0.0.1:8888".to_string(),
        vec![],
    );
    assert!(msg.is_valid(), "handshake message should be valid");
    assert!(
        msg.get_handshake_info().is_some(),
        "should carry HandshakePayload"
    );
    assert_eq!(msg.get_handshake_info().unwrap().peer_name, "Alice");
}
