use serde::{Serialize, Deserialize};
use icn_types::{Block, DID};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    // Consensus messages
    NewRound { round: u64 },
    BlockProposal { block: Block },
    Vote { round: u64, voter: DID, approve: bool },
    
    // Node messages
    PeerDiscovery { did: DID },
    PeerList { peers: Vec<DID> },
    
    // State messages  
    StateRequest { height: u64 },
    StateResponse { block: Block },
    
    // Generic messages
    Ping,
    Pong,
    Error { code: u32, message: String },
}

#[derive(Debug)]
pub struct Protocol {
    pub local_did: DID,
    pub known_peers: Vec<DID>,
}

impl Protocol {
    pub fn new(local_did: DID) -> Self {
        Self {
            local_did,
            known_peers: Vec::new(),
        }
    }
    
    pub fn handle_message(&mut self, msg: Message) -> anyhow::Result<()> {
        match msg {
            Message::PeerDiscovery { did } => {
                if !self.known_peers.contains(&did) {
                    self.known_peers.push(did);
                }
            }
            // Add handlers for other message types
            _ => ()
        }
        Ok(())
    }
}
