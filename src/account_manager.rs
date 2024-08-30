use crate::keystorage::{Error, KeyStorage, KeyStorageType};
use nostr::Keys;

pub struct AccountManager {
    pub loaded_keys: Vec<Keys>,
}

impl AccountManager {
    pub fn new() -> Self {
        Self {
            loaded_keys: Vec::new(),
        }
    }

    pub fn generate_keys(&mut self) -> Result<Keys, Error> {
        let new_keypair = Keys::generate();
        self.loaded_keys.push(new_keypair.clone());

        Ok(new_keypair)
    }

    pub fn load_keys(&mut self) -> Result<Vec<Keys>, Error> {
        let mut keys = self.get_keys()?;
        keys.append(&mut self.loaded_keys);
        keys.dedup();
        self.loaded_keys = keys.clone();

        Ok(keys)
    }

    pub fn delete_key(&mut self, key: &Keys) -> Result<(), Error> {
        self.remove_key(key)?;
        if let Some(index) = self.loaded_keys.iter().position(|k| k == key) {
            self.loaded_keys.remove(index);
        }

        Ok(())
    }

    fn get_platform_keystorage() -> KeyStorageType {
        #[cfg(target_os = "linux")]
        {
            KeyStorageType::Linux
        }

        #[cfg(not(target_os = "linux"))]
        KeyStorageType::None
    }
}

impl KeyStorage for AccountManager {
    fn get_keys(&self) -> Result<Vec<Keys>, Error> {
        Self::get_platform_keystorage().get_keys()
    }

    fn add_key(&self, key: &Keys) -> Result<(), Error> {
        Self::get_platform_keystorage().add_key(key)
    }

    fn remove_key(&self, key: &Keys) -> Result<(), Error> {
        Self::get_platform_keystorage().remove_key(key)
    }
}
