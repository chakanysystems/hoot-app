#[derive(Debug)]
pub enum Error {
    RelayNotConnected,
    SerdeJson(serde_json::Error),
    Generic(String),
    Empty,
    DecodeFailed,
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::SerdeJson(value)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::RelayNotConnected => write!(f, "Relay not connected"),
            Error::SerdeJson(err) => write!(f, "JSON serialization error: {}", err),
            Error::Generic(s) => write!(f, "{}", s),
            Error::Empty => write!(f, "Data was empty"),
            Error::DecodeFailed => write!(f, "Could not decode JSON data."),
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;
