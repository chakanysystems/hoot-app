use bitcoin::secp256k1::{Message, PublicKey, SecretKey, Keypair, schnorr::Signature, Secp256k1, hashes::{Hash, sha256}};
use serde_json::json;

mod tag;
use tag::{Tag, list::Tags};
mod id;
use id::EventId;

#[derive(Debug, Default)]
pub struct EventBuilder<'a> {
    pub created_at: Option<i64>,
    pub kind: Option<u32>,
    pub tags: Option<Tags>,
    pub content: Option<&'a str>,
}

impl EventBuilder<'_> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn kind(mut self, kind: u32) -> Self {
        self.kind = Some(kind);
        self
    }

    pub fn created_at(mut self, created_at: i64) -> Self {
        self.created_at = Some(created_at);
        self
    }

    pub fn tag(mut self, tag: Tag) -> Self {
        let tags = self.tags.get_or_insert_default();
        tags.push(tag);

        self
    }

    /// Extends the current tags.
    pub fn tags<I>(mut self, tags: I) -> Self
        where
            I: IntoIterator<Item = Tag>,
    {
        let self_tags = self.tags.get_or_insert_default();

        tags.into_iter().map(|t| self_tags.push(t));

        self
    }
}

#[derive(Debug, Clone)]
pub struct Event {
    pub id: EventId,
    pub pubkey: PublicKey,
    pub created_at: i64,
    pub kind: i32,
    pub tags: Vec<Vec<String>>,
    pub content: String,
    pub sig: Signature,
}

impl Event {
    /// Verifies the signature of the event
    pub fn verify(&self) -> bool {
        let secp = Secp256k1::verification_only();

        let message = Message::from_digest(*self.id.as_bytes());
        
        secp.verify_schnorr(&self.sig, &message, &self.pubkey.into()).is_ok()
    }

    pub fn sign_with_seckey(&mut self, seckey: &SecretKey) -> Result<(), String> {
        let secp = Secp256k1::new();

        let keypair = Keypair::from_secret_key(&secp, seckey);

        self.sign(&keypair)
    }

    /// Signs the event with the given private key
    pub fn sign(&mut self, keypair: &Keypair) -> Result<(), String> {
        let secp = Secp256k1::new();

        self.id = self.compute_id();
        let message = Message::from_digest(*self.id.as_bytes());
        self.sig = secp.sign_schnorr(&message, keypair);
        Ok(())
    }

    /// Computes the event ID
    fn compute_id(&self) -> EventId {
        let serialized = json!([
            0,
            self.pubkey,
            self.created_at,
            self.kind,
            self.tags,
            self.content
        ]);

        sha256::Hash::hash(serialized.as_str().unwrap().as_bytes()).to_string().into()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::secp256k1::{Secp256k1, SecretKey, PublicKey};

    fn create_test_event() -> Event {
        Event {
            id: EventId::default(),
            pubkey: PublicKey::from_slice(&[0; 33]).unwrap(),
            created_at: 1234567890,
            kind: 1,
            tags: vec![vec!["tag1".to_string(), "value1".to_string()]],
            content: "Test content".to_string(),
            sig: Signature::from_slice(&[0; 64]).unwrap(),
        }
    }

    #[test]
    fn test_compute_id() {
        let event = create_test_event();
        let id = event.compute_id();
        assert_ne!(id, EventId::default());
    }

    #[test]
    fn test_sign_and_verify() {
        let secp = Secp256k1::new();
        let secret_key = SecretKey::new(&mut rand::thread_rng());
        let keypair = Keypair::from_secret_key(&secp, &secret_key);

        let mut event = create_test_event();
        assert!(event.sign(&keypair).is_ok());
        assert!(event.verify());
    }

    #[test]
    fn test_sign_with_seckey() {
        let secret_key = SecretKey::new(&mut rand::thread_rng());
        let mut event = create_test_event();
        assert!(event.sign_with_seckey(&secret_key).is_ok());
        assert!(event.verify());
    }

    #[test]
    fn test_verify_invalid_signature() {
        let mut event = create_test_event();
        event.content = "Modified content".to_string();
        assert!(!event.verify());
    }
}
