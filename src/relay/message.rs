use ewebsock::WsMessage;
use nostr::types::Filter;
use nostr::Event;
use serde::de::{SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{self};

/// Messages that are client <- relay.
#[derive(Debug, Clone)]
pub enum RelayMessage {
    Event {
        subscription_id: String,
        event: Event,
    },
    Ok {
        event_id: String,
        accepted: bool,
        message: String,
    },
    Eose {
        subscription_id: String,
    },
    Closed {
        subscription_id: String,
        message: String,
    },
    Notice {
        message: String,
    },
}

impl From<WsMessage> for RelayMessage {
    fn from(value: WsMessage) -> Self {
        match value {
            WsMessage::Text(text) => {
                let parsed: RelayMessage = match serde_json::from_str(&text) {
                    Ok(p) => p,
                    Err(e) => {
                        panic!("could not parse message: {}", e);
                    }
                };
                parsed
            }
            _ => {
                panic!("Cannot parse anything but text into a RelayMessage");
            }
        }
    }
}

impl<'de> Deserialize<'de> for RelayMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RelayMessageVisitor;

        impl<'de> Visitor<'de> for RelayMessageVisitor {
            type Value = RelayMessage;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence starting with 'EVENT' or 'OK'")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let tag: &str = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;

                match tag {
                    "EVENT" => {
                        let subscription_id: String = seq
                            .next_element()?
                            .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                        let event: Event = seq
                            .next_element()?
                            .ok_or_else(|| serde::de::Error::invalid_length(2, &self))?;
                        Ok(RelayMessage::Event {
                            subscription_id,
                            event,
                        })
                    }
                    "OK" => {
                        let event_id: String = seq
                            .next_element()?
                            .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                        let accepted: bool = seq
                            .next_element()?
                            .ok_or_else(|| serde::de::Error::invalid_length(2, &self))?;
                        let message: String = seq
                            .next_element()?
                            .ok_or_else(|| serde::de::Error::invalid_length(3, &self))?;
                        Ok(RelayMessage::Ok {
                            event_id,
                            accepted,
                            message,
                        })
                    }
                    "EOSE" => {
                        let subscription_id: String = seq
                            .next_element()?
                            .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                        Ok(RelayMessage::Eose { subscription_id })
                    }
                    "CLOSED" => {
                        let subscription_id: String = seq
                            .next_element()?
                            .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                        let message: String = seq
                            .next_element()?
                            .ok_or_else(|| serde::de::Error::invalid_length(2, &self))?;
                        Ok(RelayMessage::Closed {
                            subscription_id,
                            message,
                        })
                    }
                    "NOTICE" => {
                        let message: String = seq
                            .next_element()?
                            .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                        Ok(RelayMessage::Notice { message })
                    }
                    _ => Err(serde::de::Error::invalid_value(
                        serde::de::Unexpected::Str(tag),
                        &self,
                    )),
                }
            }
        }

        deserializer.deserialize_seq(RelayMessageVisitor)
    }
}

/// Messages that are client -> relay.
#[derive(Debug, Clone)]
pub enum ClientMessage {
    Event {
        event: Event,
    },
    Req {
        subscription_id: String,
        filters: Vec<Filter>,
    },
    Close {
        subscription_id: String,
    },
}

impl From<super::Subscription> for ClientMessage {
    fn from(value: super::Subscription) -> Self {
        Self::Req {
            subscription_id: value.id,
            filters: value.filters,
        }
    }
}

impl Serialize for ClientMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ClientMessage::Event { event } => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("EVENT")?;
                seq.serialize_element(event)?;
                seq.end()
            }
            ClientMessage::Req {
                subscription_id,
                filters,
            } => {
                let mut seq = serializer.serialize_seq(Some(2 + filters.len()))?;
                seq.serialize_element("REQ")?;
                seq.serialize_element(subscription_id)?;
                for filter in filters {
                    seq.serialize_element(filter)?;
                }
                seq.end()
            }
            ClientMessage::Close { subscription_id } => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("CLOSE")?;
                seq.serialize_element(subscription_id)?;
                seq.end()
            }
        }
    }
}
