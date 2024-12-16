type Bytes = [u8; 32];

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct EventId {
    inner: Bytes,
}

impl EventId {
    #[inline]
    pub fn as_bytes(&self) -> &Bytes {
        &self.inner
    }

    #[inline]
    pub fn into_bytes(&self) -> Bytes {
        self.inner
    }
}

impl From<String> for EventId {
    fn from(s: String) -> Self {
        let mut inner = [0u8; 32];
        let bytes = s.as_bytes();
        let len = bytes.len().min(32);
        inner[..len].copy_from_slice(&bytes[..len]);
        EventId { inner }
    }
}

impl From<[u8; 32]> for EventId {
    fn from(array: [u8; 32]) -> Self {
        EventId { inner: array }
    }
}
