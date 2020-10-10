mod modules;
pub mod types;
use crate::dispatcher::types::{MessageKind, UpdateKind};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::error::Error;
use telegram_bot::{Api, ChatId, ChatMemberStatus, Message, SendMessage, Update, UserId};
use tokio::stream::StreamExt;
use tokio::sync::mpsc;

lazy_static! {
    static ref EMPTY_MODULELIST: HashMap<String, Module> = HashMap::new();
}

#[derive(Clone)]
struct Module {
    message_kind: HashMap<MessageKind, Vec<mpsc::Sender<Message>>>,
    update_kind: HashMap<UpdateKind, Vec<mpsc::Sender<telegram_bot::UpdateKind>>>,
    command: HashMap<String, Vec<mpsc::Sender<Message>>>,
}

impl Module {
    async fn handle(&mut self, update: &Update) -> Result<(), Box<dyn Error>> {
        let update_kind = UpdateKind::from(&update.kind);
        if let Some(e) = self.update_kind.get_mut(&update_kind) {
            for handler in e {
                handler.send(update.kind.clone()).await?;
            }
        };
        if let telegram_bot::UpdateKind::Message(msg) = &update.kind {
            let message_kind = MessageKind::from(&msg.kind);
            if let Some(e) = self.message_kind.get_mut(&message_kind) {
                for handler in e {
                    handler.send(msg.clone()).await?;
                }
            };
            if let telegram_bot::MessageKind::Text { data, .. } = &msg.kind {
                if let Some(command) = data.split(&[' ', '\n', '@'][..]).next() {
                    if let Some(e) = self.command.get_mut(command) {
                        for handler in e {
                            handler.send(msg.clone()).await?;
                        }
                    };
                }
            }
        }
        Ok(())
    }
}

impl<T: Subscriber> From<&T> for Module {
    fn from(sub: &T) -> Self {
        let mut res = Self {
            message_kind: HashMap::new(),
            update_kind: HashMap::new(),
            command: HashMap::new(),
        };
        for (kind, mut senders) in sub.by_message_kind().into_iter() {
            let t = res.message_kind.get_mut(&kind);
            match t {
                None => {
                    res.message_kind.insert(kind, senders);
                }
                Some(t) => t.append(&mut senders),
            }
        }
        for (kind, mut senders) in sub.by_update_kind().into_iter() {
            let t = res.update_kind.get_mut(&kind);
            match t {
                None => {
                    res.update_kind.insert(kind, senders);
                }
                Some(t) => t.append(&mut senders),
            }
        }
        for (kind, mut senders) in sub.by_command().into_iter() {
            let t = res.command.get_mut(kind);
            match t {
                None => {
                    res.command.insert(String::from(kind), senders);
                }
                Some(t) => t.append(&mut senders),
            }
        }
        res
    }
}

pub struct Dispatcher {
    db: modules::Modules,
    api: Api,
    modules: HashMap<String, Module>,
    loaded_modules: Vec<String>,
    chats: HashMap<telegram_bot::ChatId, HashMap<String, Module>>,
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
    pub fn new(api: Api) -> Dispatcher {
        let db_path = std::env::var("DISPATCHER_DB").expect("Dispatcher db path is not set");
        let mut db = modules::Modules::new(db_path).unwrap();

        Dispatcher {
            db,
            api,
            modules: HashMap::new(),
            loaded_modules: vec!(),
            chats: HashMap::new(),
        }
    }

    async fn service_cmds(
        &mut self,
        kind: &telegram_bot::MessageKind,
        chat: &ChatId,
        user: &UserId,
    ) {
        let get_member = telegram_bot::GetChatMember::new(chat, user);
        let member = crate::utils::must_send(&self.api, get_member).await;
        if !matches!(
            member.status,
            ChatMemberStatus::Administrator | ChatMemberStatus::Creator
        ) {
            return;
        }
        if let telegram_bot::MessageKind::Text { data, .. } = kind {
            let mut parts = data.split(&[' ', '\n', '@'][..]);
            if let Some(command) = parts.next() {
                match command {
                    "/enable" => {
                        if let Some(name) = parts.next() {
                            if let Some(module) = self.modules.get(name) {
                                if let Some(module_list) = self.chats.get_mut(&chat) {
                                    module_list.insert(name.to_string(), module.clone());
                                } else {
                                    let mut list = HashMap::new();
                                    list.insert(name.to_string(), module.clone());
                                    self.chats.insert(chat.clone(), list);
                                }
                                self.db.add(i64::from(chat.clone()), name);
                                crate::utils::must_send(
                                    &self.api,
                                    SendMessage::new(chat, "Module enabled"),
                                )
                                .await;
                            } else {
                                crate::utils::must_send(
                                    &self.api,
                                    SendMessage::new(chat, "Module not found"),
                                )
                                .await;
                            }
                        } else {
                            crate::utils::must_send(
                                &self.api,
                                SendMessage::new(chat, "Specify module name"),
                            )
                            .await;
                        }
                    }
                    "/disable" => {
                        if let Some(name) = parts.next() {
                            if let Some(module_list) = self.chats.get_mut(chat) {
                                module_list.remove(name);
                            }
                            self.db.rm(i64::from(chat.clone()), name);
                            crate::utils::must_send(
                                &self.api,
                                SendMessage::new(chat, "Module disabled"),
                            )
                            .await;
                        } else {
                            crate::utils::must_send(
                                &self.api,
                                SendMessage::new(chat, "Specify module name"),
                            )
                            .await;
                        }
                    }
                    "/modules" => {
                        let enabled = self.chats.get(chat).unwrap_or(&EMPTY_MODULELIST);
                        let mut msg = "`Available modules: \n".to_string();
                        for name in &self.loaded_modules {
                            let status = if enabled.contains_key(name) { "+" } else { "-" };
                            msg += format!("{} {}\n", status, &name).as_str();
                        }
                        msg += "`";
                        crate::utils::must_send(
                            &self.api,
                            SendMessage::new(chat, msg)
                                .parse_mode(telegram_bot::ParseMode::MarkdownV2),
                        )
                        .await;
                    }
                    _ => {}
                }
            }
        }
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn Error>> {
        self.loaded_modules = self
            .modules
            .iter()
            .map(|x| x.0.clone())
            .collect::<Vec<String>>();
        self.loaded_modules.sort();
        for (chat, name) in self.db.modules().unwrap() {
            let id = telegram_bot::ChatId::from(chat);
            if let Some(module) = self.modules.get(&name) {
                if let Some(module_list) = self.chats.get_mut(&id) {
                    module_list.insert(name, module.clone());
                } else {
                    let mut list = HashMap::new();
                    list.insert(name.to_string(), module.clone());
                    self.chats.insert(id, list);
                }
            } else {
                println!("Module {} not found!", &name);
            }
        }
        let mut stream = self.api.stream();
        while let Some(update) = stream.next().await {
            let update = update?;
            match &update.kind {
                telegram_bot::UpdateKind::Message(Message {
                    from, chat, kind, ..
                })
                | telegram_bot::UpdateKind::EditedMessage(Message {
                    from, chat, kind, ..
                }) => {
                    self.service_cmds(kind, &chat.id(), &from.id).await;
                    let cid = chat.id();
                    for modules in &mut self.chats.get_mut(&cid) {
                        for (_, module) in modules.iter_mut() {
                            module.handle(&update).await?;
                        }
                    }
                }
                _ => {
                    for (_, module) in self.modules.iter_mut() {
                        module.handle(&update).await?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn add_sub<T: Subscriber>(&mut self, name: String, sub: &T) {
        self.modules.insert(name, Module::from(sub));
    }
}
