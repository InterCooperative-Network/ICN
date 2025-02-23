use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::error::Error;
use libp2p::{
    floodsub::{Floodsub, FloodsubEvent, Topic},
    mdns::{Mdns, MdnsConfig, MdnsEvent},
    swarm::{SwarmBuilder, SwarmEvent},
    PeerId, Swarm, NetworkBehaviour, identity,
};
use futures::prelude::*;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Event {
    Federation(FederationEvent),
    Governance(GovernanceEvent),
    Identity(IdentityEvent),
    Reputation(ReputationEvent),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FederationEvent {
    JoinRequest { federation_id: String, member_did: String },
    // Add other federation events here
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GovernanceEvent {
    Vote { proposal_id: String, voter: String, approve: bool, zk_snark_proof: String },
    // Add other governance events here
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IdentityEvent {
    CreateIdentity { identity: String },
    // Add other identity events here
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ReputationEvent {
    ZkSnarkProofSubmitted { proof: String },
    // Add other reputation events here
}

pub struct P2PManager {
    peers: Vec<String>,
    swarm: Swarm<MyBehaviour>,
}

impl P2PManager {
    pub fn new() -> Self {
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());
        println!("Local peer id: {:?}", local_peer_id);

        let floodsub = Floodsub::new(local_peer_id.clone());
        let mdns = Mdns::new(MdnsConfig::default()).expect("Failed to create mDNS service");

        let behaviour = MyBehaviour { floodsub, mdns };

        let swarm = SwarmBuilder::new(behaviour, local_peer_id.clone())
            .executor(Box::new(|fut| {
                tokio::spawn(fut);
            }))
            .build();

        P2PManager { peers: Vec::new(), swarm }
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

    pub async fn publish(&mut self, event: Event) -> Result<(), Box<dyn Error>> {
        let topic = Topic::new("icn-events");
        let message = serde_json::to_vec(&event)?;
        self.swarm.behaviour_mut().floodsub.publish(topic, message);
        Ok(())
    }

    pub async fn subscribe(&mut self) -> Result<(), Box<dyn Error>> {
        let topic = Topic::new("icn-events");
        self.swarm.behaviour_mut().floodsub.subscribe(topic);

        loop {
            match self.swarm.next().await {
                Some(SwarmEvent::Behaviour(MyBehaviourEvent::Floodsub(FloodsubEvent::Message(message)))) => {
                    let event: Event = serde_json::from_slice(&message.data)?;
                    println!("Received event: {:?}", event);
                }
                Some(SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(MdnsEvent::Discovered(peers)))) => {
                    for (peer_id, _) in peers {
                        self.swarm.behaviour_mut().floodsub.add_node_to_partial_view(peer_id);
                    }
                }
                Some(SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(MdnsEvent::Expired(peers)))) => {
                    for (peer_id, _) in peers {
                        if !self.swarm.behaviour().mdns.has_node(&peer_id) {
                            self.swarm.behaviour_mut().floodsub.remove_node_from_partial_view(&peer_id);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

#[derive(NetworkBehaviour)]
struct MyBehaviour {
    floodsub: Floodsub,
    mdns: Mdns,
}

enum MyBehaviourEvent {
    Floodsub(FloodsubEvent),
    Mdns(MdnsEvent),
}
