use std::net::SocketAddr;
use tokio::sync::mpsc;
use quinn::{Endpoint, ServerConfig, ClientConfig};
use rustls::{Certificate, PrivateKey};
use bytes::Bytes;
use thiserror::Error;
use icn_crypto::KeyPair;

#[derive(Error, Debug)]
pub enum SDPError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Transport error: {0}")]
    TransportError(String),
    
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    
    #[error("Protocol error: {0}")]
    ProtocolError(String),
}

pub type SDPResult<T> = Result<T, SDPError>;

#[derive(Debug, Clone)]
pub struct SDPConfig {
    pub bind_address: SocketAddr,
    pub max_concurrent_streams: u32,
    pub keep_alive_interval: std::time::Duration,
    pub idle_timeout: std::time::Duration,
    pub enable_0rtt: bool,
}

impl Default for SDPConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0:0".parse().unwrap(),
            max_concurrent_streams: 100,
            keep_alive_interval: std::time::Duration::from_secs(10),
            idle_timeout: std::time::Duration::from_secs(30),
            enable_0rtt: false,
        }
    }
}

pub struct SDPEndpoint {
    endpoint: Endpoint,
    keypair: KeyPair,
    config: SDPConfig,
    message_tx: mpsc::Sender<SDPMessage>,
    message_rx: mpsc::Receiver<SDPMessage>,
}

#[derive(Debug, Clone)]
pub struct SDPMessage {
    pub destination: SocketAddr,
    pub payload: Bytes,
    pub priority: u8,
    pub reliability: ReliabilityMode,
}

#[derive(Debug, Clone, Copy)]
pub enum ReliabilityMode {
    Reliable,
    BestEffort,
    OrderedReliable,
}

impl SDPEndpoint {
    pub async fn new(config: SDPConfig, keypair: KeyPair) -> SDPResult<Self> {
        let (message_tx, message_rx) = mpsc::channel(1000);
        
        // Set up QUIC transport with our custom certificate
        let (endpoint, _server_cert) = Self::setup_transport(&config, &keypair).await?;
        
        Ok(Self {
            endpoint,
            keypair,
            config,
            message_tx,
            message_rx,
        })
    }
    
    async fn setup_transport(config: &SDPConfig, keypair: &KeyPair) -> SDPResult<(Endpoint, Certificate)> {
        // Generate self-signed certificate for QUIC
        let cert = Self::generate_certificate(keypair)?;
        
        // Configure server
        let server_config = ServerConfig::with_single_cert(
            vec![cert.clone()],
            PrivateKey(keypair.private_key.clone())
        ).map_err(|e| SDPError::EncryptionError(e.to_string()))?;
        
        // Create endpoint
        let endpoint = Endpoint::server(
            server_config,
            config.bind_address,
        ).map_err(|e| SDPError::TransportError(e.to_string()))?;
        
        Ok((endpoint, cert))
    }
    
    fn generate_certificate(keypair: &KeyPair) -> SDPResult<Certificate> {
        let cert_params = rcgen::CertificateParams::new(vec!["ICN Node".to_string()])
            .map_err(|e| SDPError::EncryptionError(e.to_string()))?;
            
        let cert = rcgen::Certificate::from_params(cert_params)
            .map_err(|e| SDPError::EncryptionError(e.to_string()))?;
            
        let cert_der = cert.serialize_der()
            .map_err(|e| SDPError::EncryptionError(e.to_string()))?;
            
        Ok(Certificate(cert_der))
    }
    
    pub async fn connect(&self, remote_addr: SocketAddr) -> SDPResult<SDPConnection> {
        // Create client config
        let client_config = Self::create_client_config()?;
        
        // Connect to remote endpoint
        let connection = self.endpoint.connect_with(client_config, remote_addr, "ICN Node")
            .map_err(|e| SDPError::ConnectionFailed(e.to_string()))?
            .await
            .map_err(|e| SDPError::ConnectionFailed(e.to_string()))?;
            
        Ok(SDPConnection {
            connection,
            message_tx: self.message_tx.clone(),
        })
    }
    
    fn create_client_config() -> SDPResult<ClientConfig> {
        let crypto = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_custom_certificate_verifier(Arc::new(AcceptAllCerts))
            .with_no_client_auth();
            
        Ok(ClientConfig::new(Arc::new(crypto)))
    }
    
    pub async fn send(&self, message: SDPMessage) -> SDPResult<()> {
        self.message_tx.send(message).await
            .map_err(|e| SDPError::TransportError(e.to_string()))?;
        Ok(())
    }
    
    pub async fn run(&mut self) -> SDPResult<()> {
        loop {
            tokio::select! {
                Some(message) = self.message_rx.recv() => {
                    self.handle_outgoing_message(message).await?;
                }
                Some(connecting) = self.endpoint.accept() => {
                    self.handle_incoming_connection(connecting).await?;
                }
            }
        }
    }
    
    async fn handle_outgoing_message(&self, message: SDPMessage) -> SDPResult<()> {
        // Get or establish connection
        let connection = self.connect(message.destination).await?;
        
        // Send message with appropriate reliability mode
        connection.send(message).await
    }
    
    async fn handle_incoming_connection(&self, connecting: quinn::Connecting) -> SDPResult<()> {
        let connection = connecting.await
            .map_err(|e| SDPError::ConnectionFailed(e.to_string()))?;
            
        // Spawn task to handle incoming streams
        tokio::spawn(async move {
            while let Ok(stream) = connection.accept_bi().await {
                // Handle incoming stream
                // ...
            }
        });
        
        Ok(())
    }
}

pub struct SDPConnection {
    connection: quinn::Connection,
    message_tx: mpsc::Sender<SDPMessage>,
}

impl SDPConnection {
    pub async fn send(&self, message: SDPMessage) -> SDPResult<()> {
        // Open bi-directional stream
        let (mut send, _recv) = self.connection.open_bi()
            .await
            .map_err(|e| SDPError::TransportError(e.to_string()))?;
            
        // Send message based on reliability mode
        match message.reliability {
            ReliabilityMode::Reliable | ReliabilityMode::OrderedReliable => {
                send.write_all(&message.payload)
                    .await
                    .map_err(|e| SDPError::TransportError(e.to_string()))?;
                    
                send.finish()
                    .await
                    .map_err(|e| SDPError::TransportError(e.to_string()))?;
            }
            ReliabilityMode::BestEffort => {
                // Best effort delivery - don't wait for acknowledgment
                let _ = send.write_all(&message.payload).await;
            }
        }
        
        Ok(())
    }
}

// Certificate verifier that accepts all certificates
struct AcceptAllCerts;

impl rustls::client::ServerCertVerifier for AcceptAllCerts {
    fn verify_server_cert(
        &self,
        _end_entity: &Certificate,
        _intermediates: &[Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
} 