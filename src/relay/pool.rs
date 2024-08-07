use crate::error::Result;
use crate::relay::{Relay, RelayStatus};
use std::collections::HashMap;

pub struct RelayPool {
    relays: HashMap<String, Relay>,
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

    pub fn try_recv(&mut self) -> Option<String> {
        for relay in &mut self.relays {
            if let Some(message) = relay.1.try_recv() {
                return Some(message);
            }
        }

        return None;
    }

    pub fn send(&mut self, message: ewebsock::WsMessage) -> Result<()> {
        for relay in &mut self.relays {
            if relay.1.status == RelayStatus::Connected {
                relay.1.send(message.clone())?;
            }
        }

        Ok(())
    }

    pub fn get_number_of_relays(&mut self) -> usize {
        self.relays.len()
    }
}
