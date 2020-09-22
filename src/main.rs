#![feature(exclusive_range_pattern)]
#![feature(test)]
extern crate test;

mod quiz;
mod dispatcher;
mod utils;
mod top;
pub(crate) mod markdown;
pub(crate) mod users;

use std::env;
use std::error::Error;
use telegram_bot::Api;
use tokio::sync::Mutex;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");
    let users = Arc::new(Mutex::new(users::Users::new(env::var("USERS_DB").expect("USERS_DB not set")).unwrap()));
    let api = Api::new(token);

    let mut disp = dispatcher::Dispatcher::new();

    let quiz = quiz::QuizModule::new(api.clone(), users.clone());
    disp.add_subscriber(quiz);

    let top = top::UserTopModule::new(api.clone(), users.clone());
    disp.add_subscriber(top);

    disp.start(api).await.unwrap();
    Ok(())
}
