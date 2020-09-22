use crate::dispatcher::Subscriber;
use std::collections::hash_map::RandomState;
use tokio::sync::{mpsc, Mutex};
use tokio::sync::mpsc::{Sender, Receiver};
use std::collections::HashMap;
use telegram_bot::{Message, Api, SendPoll, MessageId, ChatId, UserId, MessageOrChannelPost, MessageKind, SendMessage, ParseMode};
use crate::dispatcher::types::{UpdateKind};
use tokio::stream::StreamExt;
use std::sync::Arc;
use tokio::time::{Instant, Duration};
use crate::users::{Users, User};
use std::env;
use crate::quiz::quests::Quester;
use crate::quiz::messages::poll_result;
use crate::utils::must_send;

pub mod quests;
mod utils;
mod messages;

#[derive(Debug)]
pub(super) struct Poll {
    id: String,
    chat: ChatId,
    message_id: MessageId,

    correct_answers: Vec<(UserId, String)>,
    incorrect_answers: Vec<(UserId, String)>,
    correct_answer: i32,

    start: Instant
}

struct PollList {
    counts: HashMap<ChatId, u64>,
    polls: HashMap<String, Poll>
}

pub struct QuizModule {
    command: Sender<Message>,
    poll: Sender<telegram_bot::UpdateKind>,
}

impl QuizModule {
    async fn timer_loop(api: Api, quests: Arc<Mutex<Quester>>, polls: Arc<Mutex<PollList>>) {
        loop {
            tokio::time::delay_for(Duration::from_secs(2)).await;
            let mut removed_polls = vec!();
            {
                let mut polls = polls.lock().await;
                let mut remove_ids = vec!();
                for (id, poll) in polls.polls.iter() {
                    if poll.start.elapsed() > Duration::from_secs(15) {
                        remove_ids.push(id.clone());
                    }
                }

                for i in remove_ids {
                    let t = polls.polls.remove(&i).unwrap();
                    let answers = t.correct_answers.len() + t.incorrect_answers.len();
                    let chat = t.chat.clone();

                    removed_polls.push(t);

                    if answers == 0 {
                        match polls.counts.get_mut(&chat) {
                            Some(cnt) => {
                                if *cnt == 1 {
                                    *cnt = 0;
                                    continue
                                }
                            }
                            _ => {}
                        }
                    } else {
                        continue
                    }
                    unreachable!("Polls counter corrupted")
                }
            }

            for poll in removed_polls {
                let mut msg = SendMessage::new(poll.chat, poll_result(&poll));
                msg.parse_mode(ParseMode::MarkdownV2);
                must_send(&api, msg).await;

                if poll.correct_answers.len() + poll.incorrect_answers.len() != 0 {
                    loop {
                        let quest = quests.lock().await.get_quest().unwrap();
                        match QuizModule::create_poll(&api, poll.chat, quest).await {
                            Some(p) => {
                                let mut polls = polls.lock().await;
                                polls.polls.insert(p.id.clone(), p);
                                break
                            }
                            None => {}
                        }
                    }
                }
            }
        }
    }

    async fn create_poll(api: &Api, chat: ChatId, quest: quests::Question) -> Option<Poll> {
        let empty = Vec::<String>::new();
        let mut poll = SendPoll::new(chat, quest.text, empty);
        poll.quiz();
        poll.not_anonymous();
        for (i, (text, correct)) in quest.answers.into_iter().enumerate() {
            poll.add_option(text);
            if correct {
                poll.correct_option_id(i as i64);
            };
        }

        // Fuck the telegram api, it doesn't send poll update if if was closed by open_period or close_date
        poll.open_period(15);

        if let MessageOrChannelPost::Message(Message { id, kind: MessageKind::Poll { data: telegram_bot::Poll { id: poll_id, correct_option_id: Some(correct), ..} }, ..}) = api.send(poll).await.unwrap() {
            Some(Poll {
                id: poll_id,
                correct_answers: vec![],
                incorrect_answers: vec![],
                chat,
                message_id: id,
                correct_answer: correct as i32,
                start: Instant::now()
            })
        } else {
            unreachable!("Invalid message received")
        }
    }

    async fn poll_loop(mut events: Receiver<telegram_bot::UpdateKind>, users: Arc<Mutex<Users>>, list: Arc<Mutex<PollList>>) {
        while let Some(update) = events.next().await {
            match &update {
                telegram_bot::UpdateKind::PollAnswer(telegram_bot::PollAnswer { poll_id, user, option_ids }) => {
                    let mut l = list.lock().await;
                    if let Some(poll) = l.polls.get_mut(poll_id.as_str()) {
                        let last_name = match user.last_name.as_ref() {
                            Some(last_name) => {
                                last_name.clone()
                            }
                            None => {
                                String::new()
                            }
                        };
                        let username = match user.username.as_ref() {
                            Some(name) => {
                                name.clone()
                            }
                            None => {
                                String::new()
                            }
                        };
                        {
                            users.lock().await.update_user(&User {
                                uid: i64::from(user.id),
                                first_name: user.first_name.clone(),
                                last_name: last_name.clone(),
                                username
                            }).unwrap();
                        }
                        let name = match last_name.len() {
                            0 => {
                                user.first_name.clone()
                            }
                            _ => {
                                format!("{} {}", &user.first_name, last_name)
                            }
                        };
                        if poll.correct_answer == option_ids[0] as i32 {
                            users.lock().await.inc_rating(i64::from(user.id), 2).unwrap();
                            poll.correct_answers.push((user.id, name))
                        } else {
                            users.lock().await.inc_rating(i64::from(user.id), -1).unwrap();
                            poll.incorrect_answers.push((user.id, name))
                        }
                    }
                }
                _ => { }
            }
        }
    }



    async fn quiz_handler(mut events: Receiver<Message>, api: Api, polls: Arc<Mutex<PollList>>, quests: Arc<Mutex<Quester>>) {
        while let Some(msg) = events.next().await {
            {
                let mut m = polls.lock().await;
                match m.counts.get_mut(&msg.chat.id()) {
                    Some(cnt) => {
                        if *cnt > 0 {
                            continue
                        } else {
                            *cnt += 1
                        }
                    }
                    None => {
                        m.counts.insert(msg.chat.id(), 1);
                    }
                }
            }

            loop {
                let quest = quests.lock().await.get_quest().unwrap();
                match QuizModule::create_poll(&api, msg.chat.id(), quest).await {
                    Some(p) => {
                        let mut polls = polls.lock().await;
                        polls.polls.insert(p.id.clone(), p);
                        break
                    }
                    None => {}
                }
            }

        }
    }

    pub fn new(api: Api, users: Arc<Mutex<Users>>) -> QuizModule {
        let db = Arc::new(Mutex::new(quests::Quester::new(env::var("QUESTER_DB").expect("QUESTER_DB not set")).unwrap()));

        let polls = Arc::new(Mutex::new(PollList {
            polls: HashMap::new(),
            counts: HashMap::new()
        }));

        tokio::spawn(QuizModule::timer_loop(api.clone(), db.clone(), polls.clone()));

        let (poll_send, poll_recv) = mpsc::channel::<telegram_bot::UpdateKind>(1024);
        tokio::spawn(QuizModule::poll_loop(poll_recv, users.clone(), polls.clone()));

        let (command_send, command_recv) = mpsc::channel::<Message>(1024);
        tokio::spawn(QuizModule::quiz_handler(command_recv, api.clone(), polls.clone(), db.clone()));

        QuizModule {
            command: command_send,
            poll: poll_send,
        }
    }
}

impl Subscriber for QuizModule {
    fn by_update_kind(&self) -> HashMap<UpdateKind, Vec<Sender<telegram_bot::UpdateKind>>, RandomState> {
        let mut map = HashMap::new();
        map.insert(UpdateKind::PollAnswer, vec![self.poll.clone()]);
        map.insert(UpdateKind::Poll, vec![self.poll.clone()]);
        map
    }

    fn by_command(&self) -> HashMap<&str, Vec<Sender<Message>>, RandomState> {
        let mut map = HashMap::new();
        map.insert("/quiz", vec![self.command.clone()]);
        map
    }
}
