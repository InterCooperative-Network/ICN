use sodiumoxide::crypto::box_;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
    pub sender_did: String,
    pub recipient_did: String,
    pub encrypted_payload: Vec<u8>,
    pub nonce: box_::Nonce,
    pub timestamp: DateTime<Utc>,
}

pub struct MessagingService {
    keypair: box_::KeyPair,
    message_store: Arc<RwLock<HashMap<String, Vec<EncryptedMessage>>>>,
}

impl MessagingService {
    pub async fn send_message(
        &self,
        recipient_did: String,
        payload: &[u8],
        recipient_pk: box_::PublicKey
    ) -> Result<(), MessagingError> {
        let nonce = box_::gen_nonce();
        let encrypted = box_::seal(
            payload,
            &nonce,
            &recipient_pk,
            &self.keypair.secret
        );

        let message = EncryptedMessage {
            sender_did: self.did.clone(),
            recipient_did: recipient_did.clone(),
            encrypted_payload: encrypted,
            nonce,
            timestamp: Utc::now(),
        };

        let mut store = self.message_store.write().await;
        store.entry(recipient_did)
            .or_insert_with(Vec::new)
            .push(message);

        Ok(())
    }

    // ...existing code...
}
