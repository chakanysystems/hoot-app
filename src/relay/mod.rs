use tracing::{error, info, debug};
use ewebsock::{WsEvent, WsMessage};
use crate::error::{Error, Result};

mod pool;
pub use pool::RelayPool;

mod message;
pub use message::{ClientMessage, RelayMessage};

mod subscription;
pub use subscription::Subscription;

#[derive(PartialEq, Clone, Copy)]
pub enum RelayStatus {
    Connecting,
    Connected,
    Disconnected,
}

pub struct Relay {
    pub url: String,
    reader: ewebsock::WsReceiver,
    writer: ewebsock::WsSender,
    pub status: RelayStatus,
}

impl Relay {
    pub fn new_with_wakeup(
        url: impl Into<String>,
        wake_up: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        let new_url: String = url.into();
        let (sender, reciever) =
            ewebsock::connect_with_wakeup(new_url.clone(), ewebsock::Options::default(), wake_up)
                .unwrap();

        let mut relay = Self {
            url: new_url,
            reader: reciever,
            writer: sender,
            status: RelayStatus::Connecting,
        };

        relay.ping();

        relay
    }

    pub fn send(&mut self, message: WsMessage) -> Result<()> {
        if self.status != RelayStatus::Connected {
            return Err(Error::RelayNotConnected);
        }
        debug!("sending message to {}: {:?}", self.url, message);

        self.writer.send(message);
        Ok(())
    }

    pub fn try_recv(&mut self) -> Option<WsEvent> {
        if let Some(event) = self.reader.try_recv() {
            use WsEvent::*;
            match event {
                Message(_) => {
                }
                Opened => {
                    self.status = RelayStatus::Connected;
                }
                Error(ref error) => {
                    error!("error in websocket connection to {}: {}", self.url, error);
                }
                Closed => {
                    info!("connection to {} closed", self.url);
                    self.status = RelayStatus::Disconnected;
                }
            }

            return Some(event);
        }

        None
    }


    pub fn ping(&mut self) {
        let ping_msg = WsMessage::Ping(Vec::new());
        match self.send(ping_msg) {
            Ok(_) => {
                info!("Ping sent to {}", self.url);
                self.status = RelayStatus::Connected;
            }
            Err(e) => {
                error!("Error sending ping to {}: {:?}", self.url, e);
                self.status = RelayStatus::Disconnected;
            }
        }
    }
}
