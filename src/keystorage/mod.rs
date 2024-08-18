use nostr::Keys;

mod linux;
use linux::LinuxKeyStorage;

pub enum Error {
    IOError(std::io::Error)
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::IOError(value)
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(err) => write!(f, "IOError: {:?}", err),
        }
    }
}

pub enum KeyStorageType {
    None,
    #[cfg(target_os = "linux")]
    Linux,
}

pub trait KeyStorage {
    fn get_keys(&self) -> Result<Vec<Keys>, Error>;
    fn add_key(&self, key: &Keys) -> Result<(), Error>;
    fn remove_key(&self, key: &Keys) -> Result<(), Error>;
}

impl KeyStorage for KeyStorageType {
    fn add_key(&self, key: &Keys) -> Result<(), Error> {
        match self {
            Self::None => Ok(()),
            #[cfg(target_os = "linux")]
            Self::Linux => LinuxKeyStorage::new().add_key(key)
        }
    }

    fn get_keys(&self) -> Result<Vec<Keys>, Error> {
        match self {
            Self::None => Ok(Vec::new()),
            #[cfg(target_os = "linux")]
            Self::Linux => LinuxKeyStorage::new().get_keys(),
        }
    }

    fn remove_key(&self, key: &Keys) -> Result<(), Error> {
        match self {
            Self::None => Ok(()),
            #[cfg(target_os = "linux")]
            Self::Linux => LinuxKeyStorage::new().remove_key(key),
        }
    }
}
