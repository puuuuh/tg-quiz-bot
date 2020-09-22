pub mod types;

use tokio::sync::mpsc;
use std::collections::HashMap;
use crate::dispatcher::types::{MessageKind, UpdateKind};
use telegram_bot::{Message, Api};
use tokio::stream::StreamExt;
use std::error::Error;

pub struct Dispatcher {
    message_kind: HashMap<MessageKind, Vec<mpsc::Sender<Message>>>,
    update_kind: HashMap<UpdateKind, Vec<mpsc::Sender<telegram_bot::UpdateKind>>>,
    command: HashMap<String, Vec<mpsc::Sender<Message>>>
}

pub trait Subscriber {
    fn by_message_kind(&self) -> HashMap<MessageKind, Vec<mpsc::Sender<Message>>> {
        HashMap::new()
    }
    fn by_update_kind(&self) -> HashMap<UpdateKind, Vec<mpsc::Sender<telegram_bot::UpdateKind>>> {
        HashMap::new()
    }
    fn by_command(&self) -> HashMap<&str, Vec<mpsc::Sender<Message>>> {
        HashMap::new()
    }
}

impl Dispatcher {
    pub fn new() -> Dispatcher {
        Dispatcher {
            message_kind: HashMap::new(),
            update_kind: HashMap::new(),
            command: HashMap::new()
        }
    }

    pub async fn start(&mut self, api: Api) -> Result<(), Box<dyn Error>> {
        let mut stream = api.stream();
        while let Some(update) = stream.next().await {
            let update = update?;
            let update_kind = UpdateKind::from(&update.kind);
            if let Some(e) = self.update_kind.get_mut(&update_kind) {
                for handler in e {
                    handler.send(update.kind.clone()).await?;
                };
            };
            if let telegram_bot::UpdateKind::Message(msg) = update.kind {
                let message_kind = MessageKind::from(&msg.kind);
                if let Some(e) = self.message_kind.get_mut(&message_kind) {
                    for handler in e {
                        handler.send(msg.clone()).await?;
                    };
                };
                if let telegram_bot::MessageKind::Text { data, .. } = &msg.kind {
                    if let Some(command) = data.split(&[' ','\n','@'][..]).next() {
                        if let Some(e) = self.command.get_mut(command) {
                            for handler in e {
                                handler.send(msg.clone()).await?;
                            };
                        };
                    }
                }
            }
        }
        Ok(())
    }

    pub fn add_subscriber<T: Subscriber>(&mut self, sub: T) {
        for (kind, mut senders) in sub.by_message_kind().into_iter() {
            let t = self.message_kind.get_mut(&kind);
            match t {
                None => {
                    self.message_kind.insert(kind, senders);
                }
                Some(t) => {
                    t.append(&mut senders)
                }
            }
        };
        for (kind, mut senders) in sub.by_update_kind().into_iter() {
            let t = self.update_kind.get_mut(&kind);
            match t {
                None => {
                    self.update_kind.insert(kind, senders);
                }
                Some(t) => {
                    t.append(&mut senders)
                }
            }
        };
        for (kind, mut senders) in sub.by_command().into_iter() {
            let t = self.command.get_mut(kind);
            match t {
                None => {
                    self.command.insert(String::from(kind), senders);
                }
                Some(t) => {
                    t.append(&mut senders)
                }
            }
        };
    }
}