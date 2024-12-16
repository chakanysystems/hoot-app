pub mod list;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Tag {
    buf: Vec<String>,
}

impl Tag {
    #[inline]
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    #[inline]
    pub fn kind(&self) -> &str {
        &self.buf[0]
    }

    #[inline]
    pub fn content(&self) -> Option<&str> {
        self.buf.get(1).map(|s| s.as_str())
    }

    #[inline]
    pub fn len(self) -> usize {
        self.buf.len()
    }
}
