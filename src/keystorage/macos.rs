#![cfg(target_os = "macos")]

use nostr::{Keys, PublicKey, SecretKey};
use tracing::error;

use super::{Result, Error, KeyStorage};

use security_framework::item::{ItemClass, ItemSearchOptions, Limit, SearchResult};
use security_framework::passwords::{delete_generic_password, set_generic_password};

pub struct MacOSKeyStorage {
    service_name: &'static str,
}

impl MacOSKeyStorage {
    pub fn new(service_name: &'static str) -> Self {
        Self {
            service_name,
        }
    }

    fn add_key(&self, key: &Keys) -> Result<()> {
        match set_generic_password(self.service_name, &key.public_key().to_hex(), key.secret_key().unwrap().as_secret_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::Addition(key.public_key().to_hex())),
        }
    }

    fn get_pubkey_strings(&self) -> Vec<String> {
        let search_results = ItemSearchOptions::new()
            .class(ItemClass::generic_password())
            .service(self.service_name)
            .load_attributes(true)
            .limit(Limit::All)
            .search();

        let mut accounts = Vec::new();

        if let Ok(search_results) = search_results {
            for result in search_results {
                if let Some(map) = result.simplify_dict() {
                    if let Some(val) = map.get("acct") {
                        accounts.push(val.clone());
                    }
                }
            }
        }

        accounts
    }

    fn get_pubkeys(&self) -> Vec<PublicKey> {
        self.get_pubkey_strings()
            .iter_mut()
            .filter_map(|pubkey_str| PublicKey::from_hex(pubkey_str.as_str()).ok())
            .collect()
    }

    fn get_privkey_bytes_for(&self, account: &str) -> Option<Vec<u8>> {
        let search_result = ItemSearchOptions::new()
            .class(ItemClass::generic_password())
            .service(self.service_name)
            .load_data(true)
            .account(account)
            .search();

        if let Ok(results) = search_result {
            if let Some(SearchResult::Data(vec)) = results.first() {
                return Some(vec.clone());
            }
        }

        None
    }

    fn get_secret_key_for_pubkey(&self, pubkey: &PublicKey) -> Option<SecretKey> {
        if let Some(bytes) = self.get_privkey_bytes_for(pubkey.to_hex().as_str()) {
            SecretKey::from_slice(bytes.as_slice()).ok()
        } else {
            None
        }
    }

    fn get_all_keypairs(&self) -> Vec<Keys> {
        let mut keypairs: Vec<Keys> = Vec::new();
        for pubkey in self.get_pubkeys() {
            let maybe_secret = self.get_secret_key_for_pubkey(&pubkey);
            if let Some(secret) = maybe_secret {
                keypairs.push(Keys::new(secret));
            }
        }

        keypairs
    }

    fn delete_key(&self, pubkey: &PublicKey) -> Result<()> {
        match delete_generic_password(self.service_name, pubkey.to_hex().as_str()) {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("delete key error {}", e);
                Err(Error::Removal(pubkey.to_hex()))
            }
        }
    }
}

impl KeyStorage for MacOSKeyStorage {
    fn get_keys(&self) -> Result<Vec<Keys>> {
        let mmt = self.get_all_keypairs();
        Ok(mmt)
    }

    fn add_key(&self, key: &Keys) -> Result<()> {
        self.add_key(key)?;
        Ok(())
    }

    fn remove_key(&self, key: &Keys) -> Result<()> {
        self.delete_key(&key.public_key())?;
        Ok(())
    }
}
