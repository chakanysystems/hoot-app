#[derive(Debug)]
pub enum Error {
    RelayNotConnected,
    SerdeJson(serde_json::Error),
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::SerdeJson(value)
    }
}

pub type Result<T> = core::result::Result<T, Error>;
