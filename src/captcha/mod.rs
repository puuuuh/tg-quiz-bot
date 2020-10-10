use crate::dispatcher::types;
use crate::dispatcher::Subscriber;
use crate::utils::must_send;
use std::collections::HashMap;
use std::vec::Vec;
use rand::Rng;
use telegram_bot::{
    CallbackQuery, ChatId, Message, MessageId, MessageKind, MessageOrChannelPost, UpdateKind,
    UserId, CanDeleteMessage
};
use tokio::sync::mpsc;
use tokio::time::Instant;

struct UserCaptcha {
    user_id: UserId,
    chat_id: ChatId,
    date: Instant,
    task: u8,
}

pub struct Captcha {
    messages: mpsc::Sender<Message>,
    updates: mpsc::Sender<UpdateKind>,
}

impl Captcha {
    pub fn new(api: telegram_bot::Api) -> Captcha {
        let (send, recv) = mpsc::channel(1024);
        let (upd_send, upd_recv) = mpsc::channel(1024);
        tokio::spawn(Captcha::main_loop(api, recv, upd_recv));
        Captcha {
            messages: send,
            updates: upd_send,
        }
    }

    async fn main_loop(
        api: telegram_bot::Api,
        mut ch: mpsc::Receiver<Message>,
        mut upd_ch: mpsc::Receiver<UpdateKind>,
    ) {
        let mut rng = rand::rngs::OsRng::default();
        let mut pending = HashMap::<MessageId, UserCaptcha>::new();
        let mut keyboard = telegram_bot::InlineKeyboardMarkup::new();
        keyboard.add_row(vec![
            telegram_bot::InlineKeyboardButton::callback("1", "1"),
            telegram_bot::InlineKeyboardButton::callback("2", "2"),
            telegram_bot::InlineKeyboardButton::callback("3", "3"),
        ]);
        keyboard.add_row(vec![
            telegram_bot::InlineKeyboardButton::callback("4", "4"),
            telegram_bot::InlineKeyboardButton::callback("5", "5"),
            telegram_bot::InlineKeyboardButton::callback("6", "6"),
        ]);
        keyboard.add_row(vec![
            telegram_bot::InlineKeyboardButton::callback("7", "7"),
            telegram_bot::InlineKeyboardButton::callback("8", "8"),
            telegram_bot::InlineKeyboardButton::callback("9", "9"),
        ]);
        let deny_permissions = telegram_bot::ChatPermissions {
            can_send_polls: Some(false),
            can_change_info: Some(false),
            can_invite_users: Some(false),
            can_send_messages: Some(false),
            can_send_media_messages: Some(false),
            can_send_other_messages: Some(false),
            can_add_web_page_previews: Some(false),
            can_pin_messages: Some(false),
        };
        let allow_permissions = telegram_bot::ChatPermissions {
            can_send_polls: Some(true),
            can_change_info: Some(true),
            can_invite_users: Some(true),
            can_send_messages: Some(true),
            can_send_media_messages: Some(true),
            can_send_other_messages: Some(true),
            can_add_web_page_previews: Some(true),
            can_pin_messages: Some(true),
        };
        let mut timer = tokio::time::interval(tokio::time::Duration::from_secs(10));

        loop {
            tokio::select! {
                _ = timer.tick() => {
                    let mut removed = vec!();
                    for (mid, task) in &pending {
                        if task.date.elapsed() > std::time::Duration::from_secs(60) {
                            let kick = telegram_bot::KickChatMember::new(task.chat_id.clone(), task.user_id);                                                                      
                            must_send(&api, kick).await;
                            removed.push(mid.clone());
                        };
                    };
                    for mid in removed {
                        if let Some(task) = pending.remove(&mid) {
                            must_send(&api, telegram_bot::DeleteMessage::new(task.chat_id, mid)).await;
                        }
                    }
                }
                msg = ch.recv() => {
                    if let Some(Message { chat, kind: MessageKind::NewChatMembers {data, ..}, ..}) = msg {
                        for user in data {
                            let restrict_msg = telegram_bot::RestrictChatMember::new(chat.clone(), user.id, deny_permissions.clone());
                            must_send(&api, restrict_msg).await;
                            
                            let task = (rng.gen::<u8>() % 9) + 1;

                            let mut msg = telegram_bot::SendMessage::new(chat.clone(), format!("Слыш. Нажми кнопку {}, а то ебло откушу", task));
                            msg.reply_markup(keyboard.clone());

                            if let MessageOrChannelPost::Message(resp) = must_send(&api, msg).await {
                                pending.insert(resp.id, UserCaptcha {user_id: user.id, chat_id: resp.chat.id(), date: tokio::time::Instant::now(), task});
                            }
                        };
                    }
                },
                upd = upd_ch.recv() => {
                    if let Some(UpdateKind::CallbackQuery(CallbackQuery {message: Some(MessageOrChannelPost::Message(msg)), data: Some(n), from, ..})) = upd {
                        if let Some(task) = pending.get(&msg.id) {
                            if task.user_id == from.id {
                                if n.parse::<u8>().unwrap_or_default() == task.task {
                                    let restrict_msg = telegram_bot::RestrictChatMember::new(msg.chat.clone(), task.user_id, allow_permissions.clone());
                                    must_send(&api, restrict_msg).await;
                                } else {
                                    let kick = telegram_bot::KickChatMember::new(msg.chat.clone(), task.user_id);
                                    must_send(&api, kick).await;
                                }
                            } else {
                                continue
                            }
                        } else {
                            continue
                        };
                        pending.remove(&msg.id);
                        must_send(&api, msg.delete()).await;
                    }
                },
            };
        }
    }
}

impl Subscriber for Captcha {
    fn by_message_kind(
        &self,
    ) -> HashMap<crate::dispatcher::types::MessageKind, Vec<mpsc::Sender<Message>>> {
        let mut res = HashMap::new();
        res.insert(
            types::MessageKind::NewChatMembers,
            vec![self.messages.clone()],
        );
        res
    }
    fn by_update_kind(
        &self,
    ) -> HashMap<types::UpdateKind, Vec<mpsc::Sender<telegram_bot::UpdateKind>>> {
        let mut res = HashMap::new();
        res.insert(types::UpdateKind::CallbackQuery, vec![self.updates.clone()]);
        res
    }
}
