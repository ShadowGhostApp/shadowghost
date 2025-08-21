use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::Duration;
use tokio::net::UdpSocket;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPeer {
    pub id: String,
    pub address: IpAddr,
    pub port: u16,
    pub name: String,
    pub last_seen: u64,
}

pub struct NetworkDiscovery {
    local_port: u16,
    discovered_peers: HashMap<String, DiscoveredPeer>,
    is_running: bool,
}

impl NetworkDiscovery {
    pub fn new(local_port: u16) -> Self {
        Self {
            local_port,
            discovered_peers: HashMap::new(),
            is_running: false,
        }
    }

    pub async fn start_discovery(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_running {
            return Ok(());
        }

        self.is_running = true;
        Ok(())
    }

    pub fn stop_discovery(&mut self) {
        self.is_running = false;
        self.discovered_peers.clear();
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    pub fn get_discovered_peers(&self) -> Vec<DiscoveredPeer> {
        self.discovered_peers.values().cloned().collect()
    }

    pub async fn announce_presence(
        &self,
        peer_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !self.is_running {
            return Ok(());
        }

        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.set_broadcast(true)?;

        let announcement = format!(
            "SG_ANNOUNCE:{}:{}:{}",
            uuid::Uuid::new_v4(),
            peer_name,
            self.local_port
        );

        let broadcast_addr = format!("255.255.255.255:9999");
        socket
            .send_to(announcement.as_bytes(), &broadcast_addr)
            .await?;

        Ok(())
    }

    pub async fn listen_for_announcements(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.is_running {
            return Ok(());
        }

        let socket = UdpSocket::bind("0.0.0.0:9999").await?;
        let mut buf = [0u8; 1024];

        loop {
            if !self.is_running {
                break;
            }

            match tokio::time::timeout(Duration::from_secs(1), socket.recv_from(&mut buf)).await {
                Ok(Ok((len, addr))) => {
                    if let Ok(announcement) = std::str::from_utf8(&buf[..len]) {
                        self.process_announcement(announcement, addr.ip()).await;
                    }
                }
                _ => continue,
            }
        }

        Ok(())
    }

    async fn process_announcement(&mut self, announcement: &str, from_ip: IpAddr) {
        let parts: Vec<&str> = announcement.split(':').collect();
        if parts.len() >= 4 && parts[0] == "SG_ANNOUNCE" {
            let peer_id = parts[1].to_string();
            let peer_name = parts[2].to_string();
            if let Ok(port) = parts[3].parse::<u16>() {
                let peer = DiscoveredPeer {
                    id: peer_id.clone(),
                    address: from_ip,
                    port,
                    name: peer_name,
                    last_seen: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                };

                self.discovered_peers.insert(peer_id, peer);
            }
        }
    }

    pub fn cleanup_old_peers(&mut self, max_age_seconds: u64) {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.discovered_peers
            .retain(|_, peer| current_time - peer.last_seen < max_age_seconds);
    }

    pub async fn get_external_ip() -> Result<IpAddr, Box<dyn std::error::Error>> {
        let response = reqwest::get("https://api.ipify.org").await?;
        let ip_str = response.text().await?;
        Ok(ip_str.parse()?)
    }

    pub async fn test_connectivity() -> bool {
        Self::get_external_ip().await.is_ok()
    }

    pub fn get_peer_count(&self) -> usize {
        self.discovered_peers.len()
    }

    pub fn find_peer_by_name(&self, name: &str) -> Option<&DiscoveredPeer> {
        self.discovered_peers
            .values()
            .find(|peer| peer.name == name)
    }

    pub fn find_peer_by_id(&self, id: &str) -> Option<&DiscoveredPeer> {
        self.discovered_peers.get(id)
    }
}
