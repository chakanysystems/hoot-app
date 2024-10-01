use crate::error::Result;
use crate::relay::Subscription;
use crate::relay::message::ClientMessage;
use crate::relay::{Relay, RelayStatus};
use ewebsock::{WsEvent, WsMessage};
use std::collections::HashMap;
use tracing::error;

pub struct RelayPool {
    pub relays: HashMap<String, Relay>,
    pub subscriptions: HashMap<String, Subscription>,
}

impl RelayPool {
    pub fn new() -> Self {
        Self {
            relays: HashMap::new(),
            subscriptions: HashMap::new(),
        }
    }

    pub fn add_subscription(&mut self, sub: Subscription) -> Result<()> {
        {
            let cloned_sub = sub.clone();
            self.subscriptions.insert(cloned_sub.id.clone(), cloned_sub);
        }

        let client_message = ClientMessage::Req {
            subscription_id: sub.id,
            filters: sub.filters,
        };

        let payload = serde_json::to_string(&client_message)?;
        self.send(ewebsock::WsMessage::Text(payload))?;

        Ok(())
    }

    pub fn add_url(&mut self, url: String, wake_up: impl Fn() + Send + Sync + 'static) -> Result<()> {
        let relay = Relay::new_with_wakeup(url.clone(), wake_up);
        self.relays.insert(url, relay);

        Ok(())
    }

    pub fn remove_url(&mut self, url: &str) -> Option<Relay> {
        self.relays.remove(url)
    }

    pub fn try_recv(&mut self) -> Option<String> {
        for relay in self.relays.values_mut() {
            if let Some(event) = relay.try_recv() {
                use WsEvent::*;
                match event {
                    Message(message) => {
                        return self.handle_message(message);
                    }
                    Opened => {
                        for sub in self.subscriptions.clone() {
                            let client_message = ClientMessage::Req {
                                subscription_id: sub.1.id,
                                filters: sub.1.filters,
                            };

                            let payload = match serde_json::to_string(&client_message) {
                                Ok(p) => p,
                                Err(e) => {
                                    error!("could not turn subscription into json: {}", e);
                                    continue;
                                },
                            };

                            match relay.send(ewebsock::WsMessage::Text(payload)) {
                                Ok(_) => (),
                                Err(e) => error!("could not send subscription to {}: {:?}", relay.url, e),
                            };
                        }
                    }
                    _ => {
                        // we only want to know when the connection opens
                    }
                }
            }
        }
        None
    }

    fn handle_message(&mut self, message: WsMessage) -> Option<String> {
        use WsMessage::*;
        match message {
            Text(txt) => {
                return Some(txt);
            }
            Binary(..) => {
                error!("recived binary messsage, your move semisol");
            }
            Ping(d) => {
                let pong_msg = WsMessage::Pong(d);
                match self.send(pong_msg) {
                    Ok(_) => {}
                    Err(e) => error!("error when sending websocket message {:?}", e),
                }
            }
            _ => {
                // who cares
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
