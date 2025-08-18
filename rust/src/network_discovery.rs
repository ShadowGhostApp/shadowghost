use rand::Rng;
use reqwest;
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;
use tokio::net::UdpSocket;

pub struct NetworkDiscovery;

impl NetworkDiscovery {
    pub async fn get_external_ip() -> Result<IpAddr, Box<dyn std::error::Error>> {
        if let Ok(ip) = Self::get_ip_via_stun().await {
            return Ok(ip);
        }

        if let Ok(ip) = Self::get_ip_via_upnp().await {
            return Ok(ip);
        }

        Self::get_ip_via_dns_over_https().await
    }

    async fn get_ip_via_stun() -> Result<IpAddr, Box<dyn std::error::Error>> {
        let stun_servers = vec![
            "stun.l.google.com:19302",
            "stun1.l.google.com:19302",
            "stun.cloudflare.com:3478",
        ];

        for server in stun_servers {
            if let Ok(ip) = Self::query_stun_server(server).await {
                return Ok(ip);
            }
        }

        Err("All STUN servers failed".into())
    }

    async fn query_stun_server(server: &str) -> Result<IpAddr, Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        let stun_request = Self::create_stun_binding_request();

        socket.send_to(&stun_request, server).await?;

        let mut buf = [0u8; 1024];
        let (len, _) =
            tokio::time::timeout(Duration::from_secs(5), socket.recv_from(&mut buf)).await??;

        Self::parse_stun_response(&buf[..len])
    }

    fn create_stun_binding_request() -> Vec<u8> {
        let mut packet = Vec::new();
        packet.extend_from_slice(&[0x00, 0x01]);
        packet.extend_from_slice(&[0x00, 0x00]);
        packet.extend_from_slice(&[0x21, 0x12, 0xA4, 0x42]);

        let transaction_id: [u8; 12] = rand::rng().random();
        packet.extend_from_slice(&transaction_id);

        packet
    }

    fn parse_stun_response(data: &[u8]) -> Result<IpAddr, Box<dyn std::error::Error>> {
        if data.len() < 20 {
            return Err("Invalid STUN response".into());
        }

        if &data[4..8] != &[0x21, 0x12, 0xA4, 0x42] {
            return Err("Invalid STUN magic cookie".into());
        }

        let message_length = u16::from_be_bytes([data[2], data[3]]) as usize;
        let mut offset = 20;

        while offset + 4 <= data.len() && offset < 20 + message_length {
            let attr_type = u16::from_be_bytes([data[offset], data[offset + 1]]);
            let attr_length = u16::from_be_bytes([data[offset + 2], data[offset + 3]]) as usize;

            if attr_type == 0x0001 && attr_length >= 8 && offset + 12 <= data.len() {
                let family = data[offset + 5];
                if family == 0x01 {
                    let ip_bytes = &data[offset + 8..offset + 12];
                    let ip = Ipv4Addr::new(ip_bytes[0], ip_bytes[1], ip_bytes[2], ip_bytes[3]);
                    return Ok(IpAddr::V4(ip));
                }
            }

            if attr_type == 0x0020 && attr_length >= 8 && offset + 12 <= data.len() {
                let family = data[offset + 5];
                if family == 0x01 {
                    let xor_ip_bytes = &data[offset + 8..offset + 12];
                    let magic_cookie = [0x21, 0x12, 0xA4, 0x42];

                    let ip_bytes = [
                        xor_ip_bytes[0] ^ magic_cookie[0],
                        xor_ip_bytes[1] ^ magic_cookie[1],
                        xor_ip_bytes[2] ^ magic_cookie[2],
                        xor_ip_bytes[3] ^ magic_cookie[3],
                    ];

                    let ip = Ipv4Addr::new(ip_bytes[0], ip_bytes[1], ip_bytes[2], ip_bytes[3]);
                    return Ok(IpAddr::V4(ip));
                }
            }

            offset += 4 + ((attr_length + 3) & !3);
        }

        Err("No IP address found in STUN response".into())
    }

    async fn get_ip_via_upnp() -> Result<IpAddr, Box<dyn std::error::Error>> {
        let socket = std::net::UdpSocket::bind("0.0.0.0:0")?;
        socket.set_read_timeout(Some(Duration::from_secs(3)))?;

        let ssdp_request = "M-SEARCH * HTTP/1.1\r\n\
             HOST: 239.255.255.250:1900\r\n\
             MAN: \"ssdp:discover\"\r\n\
             ST: urn:schemas-upnp-org:device:InternetGatewayDevice:1\r\n\
             MX: 3\r\n\r\n";

        socket.send_to(ssdp_request.as_bytes(), "239.255.255.250:1900")?;

        let mut buf = [0u8; 1024];
        let (len, _) = socket.recv_from(&mut buf)?;
        let response = String::from_utf8_lossy(&buf[..len]);

        if let Some(location) = Self::parse_upnp_location(&response) {
            return Self::get_external_ip_from_router(&location).await;
        }

        Err("UPnP router not found".into())
    }

    fn parse_upnp_location(response: &str) -> Option<String> {
        for line in response.lines() {
            if line.to_lowercase().starts_with("location:") {
                return line
                    .split(':')
                    .skip(1)
                    .collect::<Vec<_>>()
                    .join(":")
                    .trim()
                    .to_string()
                    .into();
            }
        }
        None
    }

    async fn get_external_ip_from_router(
        location: &str,
    ) -> Result<IpAddr, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let response = client.get(location).send().await?;
        let body = response.text().await?;

        if let Some(ip_str) = Self::extract_ip_from_xml(&body) {
            return Ok(ip_str.parse()?);
        }

        Err("Failed to extract IP from router response".into())
    }

    fn extract_ip_from_xml(xml: &str) -> Option<String> {
        if let Some(start) = xml.find("<ExternalIPAddress>") {
            if let Some(end) = xml[start..].find("</ExternalIPAddress>") {
                let ip_start = start + "<ExternalIPAddress>".len();
                let ip_end = start + end;
                return Some(xml[ip_start..ip_end].to_string());
            }
        }
        None
    }

    async fn get_ip_via_dns_over_https() -> Result<IpAddr, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();

        let response = client
            .get("https://1.1.1.1/dns-query")
            .header("accept", "application/dns-json")
            .query(&[("name", "whoami.cloudflare"), ("type", "A")])
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;

        if let Some(answers) = json["Answer"].as_array() {
            for answer in answers {
                if let Some(ip_str) = answer["data"].as_str() {
                    return Ok(ip_str.parse()?);
                }
            }
        }

        Err("No IP found in DNS response".into())
    }
}
