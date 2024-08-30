use crate::error::Result;
use crate::relay::{Relay, RelayStatus};
use std::collections::HashMap;

pub struct RelayPool {
    pub relays: HashMap<String, Relay>,
}

impl RelayPool {
    pub fn new() -> Self {
        Self {
            relays: HashMap::new(),
        }
    }

    pub fn add_url(&mut self, url: String, wake_up: impl Fn() + Send + Sync + 'static) {
        let relay = Relay::new_with_wakeup(url.clone(), wake_up);
        self.relays.insert(url, relay);
    }

    pub fn remove_url(&mut self, url: &str) -> Option<Relay> {
        self.relays.remove(url)
    }

    pub fn try_recv(&mut self) -> Option<String> {
        for relay in self.relays.values_mut() {
            if let Some(message) = relay.try_recv() {
                return Some(message);
            }
        }
        None
    }

    pub fn send(&mut self, message: ewebsock::WsMessage) -> Result<()> {
        for relay in self.relays.values_mut() {
            if relay.status == RelayStatus::Connected {
                relay.send(message.clone())?;
            }
        }
        Ok(())
    }

    pub fn ping_all(&mut self) -> Result<()> {
        for relay in self.relays.values_mut() {
            relay.ping();
        }
        Ok(())
    }
}
