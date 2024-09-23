use std::collections::HashMap;
use nostr::{EventBuilder, Event, PublicKey, Kind, Tag, Keys};

pub const MAIL_EVENT_KIND: u16 = 1059;

pub struct MailMessage {
    to: Vec<PublicKey>,
    cc: Vec<PublicKey>,
    bcc: Vec<PublicKey>,
    subject: String,
    content: String,
}

impl MailMessage {
    pub fn to_events(&mut self, sending_keys: &Keys) -> HashMap<PublicKey, Event> {
        let mut pubkeys_to_send_to: Vec<PublicKey> = Vec::new();
        let mut tags: Vec<Tag> = Vec::new();

        for pubkey in &self.to {
            tags.push(Tag::public_key(*pubkey));
            pubkeys_to_send_to.push(*pubkey);
        }

        let base_event = EventBuilder::new(Kind::Custom(MAIL_EVENT_KIND), &self.content, []).to_unsigned_event(sending_keys.clone().public_key());

        let mut event_list: HashMap<PublicKey, Event> = HashMap::new();
        for pubkey in pubkeys_to_send_to {
            let wrapped_event = EventBuilder::gift_wrap(sending_keys, &pubkey, base_event.clone(), None).unwrap();
            event_list.insert(pubkey, wrapped_event);
        }

        event_list
    }
}
