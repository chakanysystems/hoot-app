use ewebsock::{WsEvent};
use tracing::{error, info};

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
        let (mut sender, reciever) = ewebsock::connect_with_wakeup(new_url.clone(), ewebsock::Options::default(), wake_up).unwrap();

        Self {
            url: new_url,
            reader: reciever,
            writer: sender,
            status: RelayStatus::Connected,
        }
    }

    pub fn try_recv(&mut self) -> Option<ewebsock::WsMessage> {
        if let Some(event) = self.reader.try_recv() {
            match event {
                WsEvent::Message(message) => {
                    return Some(message);
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
}
