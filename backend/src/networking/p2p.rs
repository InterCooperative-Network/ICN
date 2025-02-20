use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::error::Error;

pub struct P2PManager {
    peers: Vec<String>,
}

impl P2PManager {
    pub fn new() -> Self {
        P2PManager { peers: Vec::new() }
    }

    pub async fn connect(&mut self, address: &str) -> Result<(), Box<dyn Error>> {
        let stream = TcpStream::connect(address).await?;
        self.peers.push(address.to_string());
        println!("Connected to {}", address);
        Ok(())
    }

    pub async fn send_message(&self, address: &str, message: &[u8]) -> Result<(), Box<dyn Error>> {
        if let Some(peer) = self.peers.iter().find(|&&peer| peer == address) {
            let mut stream = TcpStream::connect(peer).await?;
            stream.write_all(message).await?;
            println!("Message sent to {}", address);
            Ok(())
        } else {
            Err("Peer not connected".into())
        }
    }
}
