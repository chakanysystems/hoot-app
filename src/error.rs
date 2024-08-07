#[derive(Debug)]
pub enum Error {
    RelayNotConnected,
}

pub type Result<T> = core::result::Result<T, Error>;
