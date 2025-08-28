use crate::network::types::*;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
use tokio::sync::{Mutex, RwLock};

pub struct NetworkDiscovery {
    local_port: u16,
    local_peer_id: String,
    local_peer_name: String,
    discovered_peers: Arc<RwLock<HashMap<String, DiscoveredPeer>>>,
    is_running: Arc<Mutex<bool>>,
    announcement_socket: Option<Arc<UdpSocket>>,
    listening_socket: Option<Arc<UdpSocket>>,
    discovery_handle: Option<tokio::task::JoinHandle<()>>,
    announcement_handle: Option<tokio::task::JoinHandle<()>>,
    public_key: Vec<u8>,
}

impl NetworkDiscovery {
    pub fn new(local_port: u16, peer_id: String, peer_name: String, public_key: Vec<u8>) -> Self {
        Self {
            local_port,
            local_peer_id: peer_id,
            local_peer_name: peer_name,
            discovered_peers: Arc::new(RwLock::new(HashMap::new())),
            is_running: Arc::new(Mutex::new(false)),
            announcement_socket: None,
            listening_socket: None,
            discovery_handle: None,
            announcement_handle: None,
            public_key,
        }
    }

    pub async fn start_discovery(
        &mut self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut is_running = self.is_running.lock().await;
        if *is_running {
            return Ok(());
        }

        let listening_socket = UdpSocket::bind("0.0.0.0:9999").await?;
        let announcement_socket = UdpSocket::bind("0.0.0.0:0").await?;
        announcement_socket.set_broadcast(true)?;

        self.listening_socket = Some(Arc::new(listening_socket));
        self.announcement_socket = Some(Arc::new(announcement_socket));

        let peers_clone = self.discovered_peers.clone();
        let running_clone = self.is_running.clone();
        let listening_socket_clone = self.listening_socket.as_ref().unwrap().clone();

        let discovery_handle = tokio::spawn(async move {
            Self::discovery_listener(peers_clone, running_clone, listening_socket_clone).await;
        });

        let announcement_socket_clone = self.announcement_socket.as_ref().unwrap().clone();
        let running_clone = self.is_running.clone();
        let peer_id = self.local_peer_id.clone();
        let peer_name = self.local_peer_name.clone();
        let port = self.local_port;
        let public_key = self.public_key.clone();

        let announcement_handle = tokio::spawn(async move {
            Self::announcement_broadcaster(
                announcement_socket_clone,
                running_clone,
                peer_id,
                peer_name,
                port,
                public_key,
            )
            .await;
        });

        self.discovery_handle = Some(discovery_handle);
        self.announcement_handle = Some(announcement_handle);

        *is_running = true;
        Ok(())
    }

    pub async fn stop_discovery(&mut self) {
        let mut is_running = self.is_running.lock().await;
        *is_running = false;

        if let Some(handle) = self.discovery_handle.take() {
            handle.abort();
        }

        if let Some(handle) = self.announcement_handle.take() {
            handle.abort();
        }

        self.discovered_peers.write().await.clear();
    }

    async fn discovery_listener(
        peers: Arc<RwLock<HashMap<String, DiscoveredPeer>>>,
        is_running: Arc<Mutex<bool>>,
        socket: Arc<UdpSocket>,
    ) {
        let mut buffer = [0u8; 2048];

        while *is_running.lock().await {
            match tokio::time::timeout(Duration::from_secs(1), socket.recv_from(&mut buffer)).await
            {
                Ok(Ok((len, addr))) => {
                    if let Ok(message_str) = std::str::from_utf8(&buffer[..len]) {
                        if let Ok(announcement) =
                            serde_json::from_str::<AnnouncementMessage>(message_str)
                        {
                            Self::process_announcement(announcement, addr.ip(), peers.clone())
                                .await;
                        }
                    }
                }
                Ok(Err(e)) => eprintln!("Discovery receive error: {}", e),
                Err(_) => continue,
            }
        }
    }

    async fn announcement_broadcaster(
        socket: Arc<UdpSocket>,
        is_running: Arc<Mutex<bool>>,
        peer_id: String,
        peer_name: String,
        port: u16,
        public_key: Vec<u8>,
    ) {
        let mut interval = tokio::time::interval(Duration::from_secs(30));

        while *is_running.lock().await {
            interval.tick().await;

            let announcement = AnnouncementMessage {
                peer_id: peer_id.clone(),
                peer_name: peer_name.clone(),
                port,
                public_key: public_key.clone(),
                protocol_version: 1,
                capabilities: vec!["chat".to_string(), "file_transfer".to_string()],
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            };

            if let Ok(message_json) = serde_json::to_string(&announcement) {
                let broadcast_addresses = ["255.255.255.255:9999", "224.0.0.1:9999"];

                for addr in &broadcast_addresses {
                    let _ = socket.send_to(message_json.as_bytes(), addr).await;
                }

                for subnet in &[
                    "192.168.1.255:9999",
                    "10.0.0.255:9999",
                    "172.16.255.255:9999",
                ] {
                    let _ = socket.send_to(message_json.as_bytes(), subnet).await;
                }
            }
        }
    }

    async fn process_announcement(
        announcement: AnnouncementMessage,
        from_ip: IpAddr,
        peers: Arc<RwLock<HashMap<String, DiscoveredPeer>>>,
    ) {
        let discovered_peer = DiscoveredPeer {
            id: announcement.peer_id.clone(),
            address: from_ip,
            port: announcement.port,
            name: announcement.peer_name,
            last_seen: announcement.timestamp,
            public_key: announcement.public_key,
            protocol_version: announcement.protocol_version,
            capabilities: announcement.capabilities,
        };

        let mut peers_map = peers.write().await;
        peers_map.insert(announcement.peer_id, discovered_peer);
    }

    pub fn is_running(&self) -> bool {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async { *self.is_running.lock().await })
        })
    }

    pub async fn get_discovered_peers(&self) -> Vec<DiscoveredPeer> {
        self.discovered_peers
            .read()
            .await
            .values()
            .cloned()
            .collect()
    }

    pub async fn announce_presence(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(socket) = &self.announcement_socket {
            let announcement = AnnouncementMessage {
                peer_id: self.local_peer_id.clone(),
                peer_name: self.local_peer_name.clone(),
                port: self.local_port,
                public_key: self.public_key.clone(),
                protocol_version: 1,
                capabilities: vec!["chat".to_string(), "file_transfer".to_string()],
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            };

            let message_json = serde_json::to_string(&announcement)?;
            socket
                .send_to(message_json.as_bytes(), "255.255.255.255:9999")
                .await?;
        }
        Ok(())
    }

    pub async fn cleanup_old_peers(&self, max_age_seconds: u64) {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut peers = self.discovered_peers.write().await;

        peers.retain(|_, peer| current_time - peer.last_seen < max_age_seconds);
    }

    pub async fn get_external_ip() -> Result<IpAddr, Box<dyn std::error::Error + Send + Sync>> {
        let services = [
            "https://api.ipify.org",
            "https://ipinfo.io/ip",
            "https://httpbin.org/ip",
        ];

        for service in &services {
            match reqwest::get(*service).await {
                Ok(response) => {
                    if let Ok(text) = response.text().await {
                        if service.contains("httpbin") {
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                                if let Some(origin) = json.get("origin") {
                                    if let Some(ip_str) = origin.as_str() {
                                        let ip_parts: Vec<&str> = ip_str.split(',').collect();
                                        if let Ok(ip) = ip_parts[0].trim().parse::<IpAddr>() {
                                            return Ok(ip);
                                        }
                                    }
                                }
                            }
                        } else {
                            let cleaned_text = text.trim();
                            if let Ok(ip) = cleaned_text.parse::<IpAddr>() {
                                return Ok(ip);
                            }
                        }
                    }
                }
                Err(_) => continue,
            }
        }

        Err("Failed to determine external IP".into())
    }

    pub async fn test_connectivity() -> bool {
        Self::get_external_ip().await.is_ok()
    }

    pub async fn get_peer_count(&self) -> usize {
        self.discovered_peers.read().await.len()
    }

    pub async fn find_peer_by_name(&self, name: &str) -> Option<DiscoveredPeer> {
        let peers = self.discovered_peers.read().await;
        peers.values().find(|peer| peer.name == name).cloned()
    }

    pub async fn find_peer_by_id(&self, id: &str) -> Option<DiscoveredPeer> {
        let peers = self.discovered_peers.read().await;
        peers.get(id).cloned()
    }

    pub async fn get_peers_by_capability(&self, capability: &str) -> Vec<DiscoveredPeer> {
        let peers = self.discovered_peers.read().await;
        peers
            .values()
            .filter(|peer| peer.capabilities.contains(&capability.to_string()))
            .cloned()
            .collect()
    }

    pub async fn get_discovery_statistics(&self) -> DiscoveryStatistics {
        let peers = self.discovered_peers.read().await;
        let total_peers = peers.len();
        let active_peers = peers
            .values()
            .filter(|p| {
                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                current_time - p.last_seen < 300
            })
            .count();

        let capabilities: std::collections::HashSet<String> = peers
            .values()
            .flat_map(|p| p.capabilities.iter())
            .cloned()
            .collect();

        DiscoveryStatistics {
            total_discovered: total_peers,
            active_peers,
            unique_capabilities: capabilities.len(),
            discovery_uptime: 0,
            last_discovery: chrono::Utc::now(),
        }
    }
}
