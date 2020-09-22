mod ranks;

use tokio::sync::mpsc::{Receiver, Sender};
use telegram_bot::{Api, Message, SendMessage, ParseMode};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tokio::stream::StreamExt;
use crate::utils::must_send;
use std::collections::HashMap;
use crate::markdown;
use crate::top::ranks::score_to_rank;
use crate::users::Users;
use crate::dispatcher::Subscriber;
use std::collections::hash_map::RandomState;

pub struct UserTopModule {
    top: Sender<Message>,
}

impl UserTopModule {
    pub fn new(api: Api, users: Arc<Mutex<Users>>) -> UserTopModule {
        let (top_send, top_recv) = mpsc::channel::<Message>(1024);
        tokio::spawn(UserTopModule::top_handler(top_recv, api.clone(), users.clone()));

        UserTopModule {
            top: top_send,
        }
    }

    async fn top_handler(mut events: Receiver<Message>, api: Api, users: Arc<Mutex<Users>>) {
        while let Some(msg) = events.next().await {
            let top = {
                users.lock().await.get_top(20).unwrap()
            };

            let mut pos = 1;
            let mut data = String::from("Top 10: ");
            for u in top {
                if u.1 < 0 {
                    data += &format!("\n{} {}: \\{}, {}", pos, markdown::escape(&markdown::full_name(&u.0.first_name, &u.0.last_name)), u.1, markdown::bold(score_to_rank(u.1)));
                } else {
                    data += &format!("\n{} {}: {}, {}", pos, markdown::escape(&markdown::full_name(&u.0.first_name, &u.0.last_name)), u.1, markdown::bold(score_to_rank(u.1)));
                }
                pos += 1;
            }
            let c = msg.chat;
            let mut msg = SendMessage::new(&c, &data);
            msg.parse_mode(ParseMode::MarkdownV2);
            must_send(&api, msg).await;
        }
    }
}

impl Subscriber for UserTopModule {
    fn by_command(&self) -> HashMap<&str, Vec<Sender<Message>>, RandomState> {
        let mut map = HashMap::new();
        map.insert("/top", vec![self.top.clone()]);
        map
    }
}

