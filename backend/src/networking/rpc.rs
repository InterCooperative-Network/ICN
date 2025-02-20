use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::error::Error;

pub struct RpcManager;

impl RpcManager {
    pub fn new() -> Self {
        RpcManager
    }

    pub async fn handle_rpc(&self, address: &str) -> Result<(), Box<dyn Error>> {
        let listener = TcpListener::bind(address).await?;
        println!("RPC server listening on {}", address);

        loop {
            let (mut socket, _) = listener.accept().await?;
            tokio::spawn(async move {
                let mut buffer = [0; 1024];
                match socket.read(&mut buffer).await {
                    Ok(_) => {
                        let response = b"RPC response";
                        if let Err(e) = socket.write_all(response).await {
                            eprintln!("Failed to write to socket: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to read from socket: {}", e);
                    }
                }
            });
        }
    }
}
