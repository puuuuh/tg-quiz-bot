use crate::dispatcher::Subscriber;
use tokio::sync::mpsc;
use telegram_bot::{Message, MessageKind};
use crate::dispatcher::types;
use std::collections::{HashMap};
use std::vec::Vec;

pub struct Antimoon {
    messages: mpsc::Sender<Message>    
}

impl Antimoon {
    pub fn new(api: telegram_bot::Api) -> Antimoon {
        let (send, recv) = mpsc::channel(1024);
        tokio::spawn(Antimoon::main_loop(api, recv));
        Antimoon {
            messages: send,
        }
    }

    async fn main_loop(api: telegram_bot::Api, mut ch: mpsc::Receiver<Message>) {
        while let Some(Message {id, chat, kind: MessageKind::Text{data, ..}, ..}) = ch.recv().await {
            if data.contains("🌚") {
                api.send(telegram_bot::DeleteMessage::new(chat, id)).await.unwrap();
            }
        }
    }
}

impl Subscriber for Antimoon {
    fn by_message_kind(&self) -> HashMap<crate::dispatcher::types::MessageKind, Vec<mpsc::Sender<Message>>> {
        let mut res = HashMap::new();
        res.insert(types::MessageKind::Text, vec![self.messages.clone()]);
        res
    }
}
