use ewebsock::{WsEvent, WsMessage};
use tracing::{error, info};

use crate::error::{Error, Result};

mod pool;
pub use pool::RelayPool;

mod message;
pub use message::{ClientMessage, RelayMessage};

mod subscription;
pub use subscription::Subscription;

#[derive(PartialEq)]
pub enum RelayStatus {
    Connecting,
    Connected,
    Disconnected,
}

pub struct Relay {
    url: String,
    reader: ewebsock::WsReceiver,
    writer: ewebsock::WsSender,
    status: RelayStatus,
}

impl Relay {
    pub fn new_with_wakeup(url: impl Into<String>, wake_up: impl Fn() + Send + Sync + 'static) -> Self {
        let new_url: String = url.into();
        let (sender, reciever) = ewebsock::connect_with_wakeup(new_url.clone(), ewebsock::Options::default(), wake_up).unwrap();

        Self {
            url: new_url,
            reader: reciever,
            writer: sender,
            status: RelayStatus::Connected,
        }
    }

    pub fn send(&mut self, message: WsMessage) -> Result<()> {
        if self.status != RelayStatus::Connected {
            return Err(Error::RelayNotConnected);
        }

        self.writer.send(message);
        Ok(())
    }

    pub fn try_recv(&mut self) -> Option<String> {
        if let Some(event) = self.reader.try_recv() {
            match event {
                WsEvent::Message(message) => {
                    return self.handle_message(message);
                },
                WsEvent::Opened => {
                    self.status = RelayStatus::Connected;
                },
                WsEvent::Error(error) => {
                    error!("error in websocket connection to {}: {}", self.url, error);
                },
                WsEvent::Closed => {
                    info!("connection to {} closed", self.url);
                    self.status = RelayStatus::Disconnected;
                }
            }
        }

        return None;
    }

    fn handle_message(&mut self, message: WsMessage) -> Option<String> {
        match message {
            WsMessage::Text(txt) => {
                return Some(txt);
            },
            WsMessage::Binary(..) => {
                error!("recived binary messsage, your move semisol");
            },
            WsMessage::Ping(d) => {
                let pong_msg = WsMessage::Pong(d);
                match self.send(pong_msg) {
                    Ok(_) => {},
                    Err(e) => error!("error when sending websocket message {:?}", e)
                }
            },
            WsMessage::Pong(..) => {
                // ??
            },
            WsMessage::Unknown(..) => {
                // who cares
            },
        }

        None
    }
}