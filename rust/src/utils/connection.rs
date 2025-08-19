use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub struct ConnectionUtils;

impl ConnectionUtils {
    pub async fn verify_connection_active(address: &str) -> bool {
        match Self::ping_address(address).await {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    async fn ping_address(address: &str) -> Result<(), Box<dyn std::error::Error>> {
        let stream_result =
            tokio::time::timeout(Duration::from_secs(3), TcpStream::connect(address)).await;

        let mut stream = match stream_result {
            Ok(Ok(s)) => s,
            Ok(Err(_)) => return Err("Connection failed".into()),
            Err(_) => return Err("Connection timeout".into()),
        };

        let ping_data = b"PING";

        match tokio::time::timeout(Duration::from_secs(2), stream.write_all(ping_data)).await {
            Ok(Ok(_)) => {
                let _ = stream.flush().await;

                let mut buffer = [0; 1024];
                match tokio::time::timeout(Duration::from_secs(3), stream.read(&mut buffer)).await {
                    Ok(Ok(n)) if n > 0 => {
                        if buffer[..4] == *b"PONG" {
                            Ok(())
                        } else {
                            Err("Invalid response".into())
                        }
                    }
                    _ => Err("No response".into()),
                }
            }
            _ => Err("Failed to send ping".into()),
        }
    }

    pub fn parse_address(address: &str) -> Result<(String, u16), Box<dyn std::error::Error>> {
        if let Some(colon_pos) = address.rfind(':') {
            let host = &address[..colon_pos];
            let port_str = &address[colon_pos + 1..];

            let port: u16 = port_str
                .parse()
                .map_err(|_| format!("Invalid port: {}", port_str))?;

            Ok((host.to_string(), port))
        } else {
            Err("Address must contain port".into())
        }
    }

    pub fn validate_peer_address(address: &str) -> bool {
        if address.is_empty() {
            return false;
        }

        if !address.contains(':') {
            return false;
        }

        match Self::parse_address(address) {
            Ok((host, port)) => !host.is_empty() && port > 0,
            Err(_) => false,
        }
    }

    pub async fn test_connection_reliability(address: &str, attempts: u32) -> f64 {
        let mut successful = 0;

        for _ in 0..attempts {
            if Self::verify_connection_active(address).await {
                successful += 1;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        successful as f64 / attempts as f64
    }

    pub fn is_local_address(address: &str) -> bool {
        address.starts_with("127.0.0.1")
            || address.starts_with("localhost")
            || address.starts_with("::1")
    }

    pub fn normalize_address(address: &str) -> String {
        if address.starts_with("localhost:") {
            address.replace("localhost", "127.0.0.1")
        } else {
            address.to_string()
        }
    }
}

pub mod message_utils {
    use crate::network::{ChatMessage, DeliveryStatus};

    pub fn filter_delivered_messages(messages: &[ChatMessage]) -> Vec<&ChatMessage> {
        messages
            .iter()
            .filter(|m| matches!(m.delivery_status, DeliveryStatus::Delivered))
            .collect()
    }

    pub fn filter_failed_messages(messages: &[ChatMessage]) -> Vec<&ChatMessage> {
        messages
            .iter()
            .filter(|m| matches!(m.delivery_status, DeliveryStatus::Failed))
            .collect()
    }

    pub fn filter_pending_messages(messages: &[ChatMessage]) -> Vec<&ChatMessage> {
        messages
            .iter()
            .filter(|m| matches!(m.delivery_status, DeliveryStatus::Pending))
            .collect()
    }

    pub fn get_delivery_success_rate(messages: &[ChatMessage]) -> f64 {
        if messages.is_empty() {
            return 0.0;
        }

        let delivered_count = filter_delivered_messages(messages).len();
        delivered_count as f64 / messages.len() as f64
    }

    pub fn has_undelivered_messages(messages: &[ChatMessage]) -> bool {
        messages.iter().any(|m| {
            matches!(
                m.delivery_status,
                DeliveryStatus::Pending | DeliveryStatus::Failed
            )
        })
    }
}

pub mod validation {
    pub fn validate_message_content(content: &str) -> Result<(), String> {
        if content.is_empty() {
            return Err("Message content cannot be empty".to_string());
        }

        if content.len() > 10000 {
            return Err("Message content too long".to_string());
        }

        Ok(())
    }

    pub fn validate_contact_name(name: &str) -> Result<(), String> {
        if name.is_empty() {
            return Err("Contact name cannot be empty".to_string());
        }

        if name.len() > 50 {
            return Err("Contact name too long".to_string());
        }

        if name.contains(&['\n', '\r', '\t']) {
            return Err("Contact name contains invalid characters".to_string());
        }

        Ok(())
    }

    pub fn validate_sg_link_format(link: &str) -> Result<(), String> {
        if !link.starts_with("sg://") {
            return Err("SG link must start with sg://".to_string());
        }

        if link.len() < 10 {
            return Err("SG link too short".to_string());
        }

        let encoded_part = &link[5..];
        if encoded_part.is_empty() {
            return Err("SG link missing data".to_string());
        }

        Ok(())
    }
}
