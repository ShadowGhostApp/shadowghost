use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::network::protocol::{ProtocolMessage, MAX_MESSAGE_SIZE};
use super::node::P2pError;

/// Serialize `msg` and write `[u32 BE length][JSON bytes]` to `writer`.
///
/// Follows Briar's Bramble framing convention: a fixed-size length prefix
/// avoids delimiter scanning and allows O(1) frame extraction.
pub async fn write_frame<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    msg: &ProtocolMessage,
) -> Result<(), P2pError> {
    let bytes = msg
        .to_bytes()
        .map_err(|e| P2pError::Protocol(e.to_string()))?;

    if bytes.len() > MAX_MESSAGE_SIZE {
        return Err(P2pError::MessageTooLarge { size: bytes.len() });
    }

    writer.write_u32(bytes.len() as u32).await?;
    writer.write_all(&bytes).await?;
    writer.flush().await?;
    Ok(())
}

/// Read one `[u32 BE length][JSON bytes]` frame and deserialize it.
///
/// Returns `P2pError::ConnectionClosed` on clean EOF so callers can
/// distinguish peer disconnect from actual protocol errors.
pub async fn read_frame<R: AsyncReadExt + Unpin>(
    reader: &mut R,
) -> Result<ProtocolMessage, P2pError> {
    let len = match reader.read_u32().await {
        Ok(l) => l,
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
            return Err(P2pError::ConnectionClosed);
        }
        Err(e) => return Err(P2pError::Io(e)),
    };

    if len as usize > MAX_MESSAGE_SIZE {
        return Err(P2pError::MessageTooLarge { size: len as usize });
    }

    let mut buf = vec![0u8; len as usize];
    reader.read_exact(&mut buf).await.map_err(|e| {
        if e.kind() == io::ErrorKind::UnexpectedEof {
            P2pError::ConnectionClosed
        } else {
            P2pError::Io(e)
        }
    })?;

    ProtocolMessage::from_bytes(&buf).map_err(|e| P2pError::Protocol(e.to_string()))
}
