mod tag;
use tag::{Tag, list::Tags};

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
use bitcoin::{
    hex,
    secp256k1::{Message, SecretKey, Secp256k1, hashes::{Hash, sha256}}
};
use serde_json::json;

#[derive(Debug, Clone)]
pub struct Event {
    pub id: String,
    pub pubkey: String,
    pub created_at: i64,
    pub kind: i32,
    pub tags: Vec<Vec<String>>,
    pub content: String,
    pub sig: String,
}

impl Event {
    /// Verifies the signature of the event
    pub fn verify(&self) -> bool {
        let secp = Secp256k1::verification_only();

        // Get the event id as message
        let message = match Message::from_digest(self.id.as_bytes()) {
            Ok(msg) => msg,
            Err(_) => return false,
        };

        // Parse the public key
        let pubkey = match PublicKey::from_slice(&hex::decode(&self.pubkey).unwrap()) {
            Ok(key) => key,
            Err(_) => return false,
        };

        // Parse the signature
        let sig = match Signature::from_slice(&hex::decode(&self.sig).unwrap()) {
            Ok(s) => s,
            Err(_) => return false,
        };

        // Verify the signature
        secp.verify(&message, &sig, &pubkey).is_ok()
    }

    /// Signs the event with the given private key
    pub fn sign(&mut self, seckey: &[u8]) -> Result<(), String> {
        let secp = Secp256k1::new();

        self.compute_id();

        // Create the message from the id
        let message = match Message::from_digest(&hex::decode(&self.id).unwrap())

        // Parse the secret key
        let secret_key = SecretKey::from_slice(seckey).map_err(|e| e.to_string())?;

        // Sign the message
        let sig = secp.sign_ecdsa(&message, &secret_key);

        // Convert signature to hex string
        self.sig = hex::encode(sig.serialize_compact());

        Ok(())
    }

    /// Computes the event ID
    fn compute_id(mut self) {
        let serialized = json!([
            0,
            self.pubkey,
            self.created_at,
            self.kind,
            self.tags,
            self.content
        ]);

        // lol
        self.id = sha256::Hash::hash(serialized.as_str().unwrap().as_bytes()).to_string();
    }
}

