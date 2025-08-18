use rand::Rng;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub struct TlsMaskedConnection {
    stream: TcpStream,
    handshake_complete: bool,
    domain_mask: String,
}

impl TlsMaskedConnection {
    pub async fn connect_as_client(
        target: &str,
        mask_domain: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let stream = TcpStream::connect(&format!("{}:443", target)).await?;
        let mut conn = Self {
            stream,
            handshake_complete: false,
            domain_mask: mask_domain.to_string(),
        };

        conn.perform_fake_tls_handshake().await?;
        Ok(conn)
    }

    pub async fn accept_as_server(
        stream: TcpStream,
        mask_domain: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut conn = Self {
            stream,
            handshake_complete: false,
            domain_mask: mask_domain.to_string(),
        };

        conn.handle_fake_tls_handshake().await?;
        Ok(conn)
    }

    async fn perform_fake_tls_handshake(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let client_hello = self.create_fake_client_hello()?;
        self.stream.write_all(&client_hello).await?;

        let mut response = vec![0u8; 4096];
        let n = self.stream.read(&mut response).await?;

        if !self.validate_server_hello(&response[..n])? {
            return Err("Invalid TLS handshake response".into());
        }

        let client_finish = self.create_client_finish()?;
        self.stream.write_all(&client_finish).await?;

        self.handshake_complete = true;
        Ok(())
    }

    async fn handle_fake_tls_handshake(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = vec![0u8; 1024];
        let n = self.stream.read(&mut buffer).await?;

        if !self.validate_client_hello(&buffer[..n])? {
            return Err("Invalid Client Hello".into());
        }

        let server_response = self.create_fake_server_hello()?;
        self.stream.write_all(&server_response).await?;

        let mut finish_buffer = vec![0u8; 1024];
        let _n = self.stream.read(&mut finish_buffer).await?;

        self.handshake_complete = true;
        Ok(())
    }

    fn create_fake_client_hello(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut packet = Vec::new();
        packet.push(0x16);
        packet.extend_from_slice(&[0x03, 0x03]);

        let mut handshake = Vec::new();
        handshake.push(0x01);

        let mut client_hello = Vec::new();
        client_hello.extend_from_slice(&[0x03, 0x03]);

        let random: [u8; 32] = rand::rng().random();
        client_hello.extend_from_slice(&random);
        client_hello.push(0x00);

        client_hello.extend_from_slice(&[0x00, 0x02]);
        client_hello.extend_from_slice(&[0x00, 0x35]);

        client_hello.push(0x01);
        client_hello.push(0x00);

        let mut extensions = Vec::new();
        let sni_ext = self.create_sni_extension()?;
        extensions.extend_from_slice(&sni_ext);

        let alpn_ext = self.create_alpn_extension()?;
        extensions.extend_from_slice(&alpn_ext);

        if !extensions.is_empty() {
            client_hello.extend_from_slice(&(extensions.len() as u16).to_be_bytes());
            client_hello.extend_from_slice(&extensions);
        } else {
            client_hello.extend_from_slice(&[0x00, 0x00]);
        }

        let length = client_hello.len() as u32;
        handshake.extend_from_slice(&length.to_be_bytes()[1..4]);
        handshake.extend_from_slice(&client_hello);

        packet.extend_from_slice(&(handshake.len() as u16).to_be_bytes());
        packet.extend_from_slice(&handshake);

        Ok(packet)
    }

    fn create_sni_extension(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut ext = Vec::new();
        ext.extend_from_slice(&[0x00, 0x00]);

        let mut sni_data = Vec::new();

        let list_length = 3 + self.domain_mask.len();
        sni_data.extend_from_slice(&(list_length as u16).to_be_bytes());
        sni_data.push(0x00);
        sni_data.extend_from_slice(&(self.domain_mask.len() as u16).to_be_bytes());
        sni_data.extend_from_slice(self.domain_mask.as_bytes());

        ext.extend_from_slice(&(sni_data.len() as u16).to_be_bytes());
        ext.extend_from_slice(&sni_data);

        Ok(ext)
    }

    fn create_alpn_extension(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut ext = Vec::new();
        ext.extend_from_slice(&[0x00, 0x10]);

        let mut alpn_data = Vec::new();
        let protocols = vec!["h2", "http/1.1"];

        let mut protocol_list = Vec::new();
        for proto in protocols {
            protocol_list.push(proto.len() as u8);
            protocol_list.extend_from_slice(proto.as_bytes());
        }

        alpn_data.extend_from_slice(&(protocol_list.len() as u16).to_be_bytes());
        alpn_data.extend_from_slice(&protocol_list);

        ext.extend_from_slice(&(alpn_data.len() as u16).to_be_bytes());
        ext.extend_from_slice(&alpn_data);

        Ok(ext)
    }

    pub async fn send_p2p_message(
        &mut self,
        message: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !self.handshake_complete {
            return Err("TLS handshake not complete".into());
        }

        let http2_frame = self.wrap_as_http2_data(message)?;
        let tls_record = self.wrap_as_tls_application_data(&http2_frame)?;

        self.stream.write_all(&tls_record).await?;
        Ok(())
    }

    pub async fn read_p2p_message(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        if !self.handshake_complete {
            return Err("TLS handshake not complete".into());
        }

        let mut header = [0u8; 5];
        self.stream.read_exact(&mut header).await?;

        if header[0] != 0x17 {
            return Err("Expected TLS Application Data".into());
        }

        let length = u16::from_be_bytes([header[3], header[4]]) as usize;
        let mut payload = vec![0u8; length];
        self.stream.read_exact(&mut payload).await?;

        let p2p_data = self.unwrap_from_http2_data(&payload)?;
        Ok(p2p_data)
    }

    fn wrap_as_http2_data(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut frame = Vec::new();

        let length = data.len() as u32;
        frame.extend_from_slice(&length.to_be_bytes()[1..4]);
        frame.push(0x00);
        frame.push(0x01);

        let stream_id: u32 = rand::rng().random_range(1..=0x7FFFFFFF);
        frame.extend_from_slice(&stream_id.to_be_bytes());
        frame.extend_from_slice(data);

        Ok(frame)
    }

    fn unwrap_from_http2_data(
        &self,
        frame_data: &[u8],
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        if frame_data.len() < 9 {
            return Err("Invalid HTTP/2 frame".into());
        }

        let length = u32::from_be_bytes([0, frame_data[0], frame_data[1], frame_data[2]]) as usize;
        let frame_type = frame_data[3];

        if frame_type != 0x00 {
            return Err("Expected HTTP/2 DATA frame".into());
        }

        if frame_data.len() < 9 + length {
            return Err("Incomplete HTTP/2 frame".into());
        }

        Ok(frame_data[9..9 + length].to_vec())
    }

    fn wrap_as_tls_application_data(
        &self,
        data: &[u8],
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut record = Vec::new();
        record.push(0x17);
        record.extend_from_slice(&[0x03, 0x03]);
        record.extend_from_slice(&(data.len() as u16).to_be_bytes());
        record.extend_from_slice(data);
        Ok(record)
    }

    fn create_fake_server_hello(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(vec![
            0x16, 0x03, 0x03, 0x00, 0x4A, 0x02, 0x00, 0x00, 0x46, 0x03, 0x03,
        ])
    }

    fn create_client_finish(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(vec![
            0x14, 0x03, 0x03, 0x00, 0x01, 0x01, 0x16, 0x03, 0x03, 0x00, 0x10,
        ])
    }

    fn validate_client_hello(&self, _data: &[u8]) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(true)
    }

    fn validate_server_hello(&self, _data: &[u8]) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(true)
    }
}
