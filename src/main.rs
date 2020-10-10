#![feature(exclusive_range_pattern)]
#![feature(test)]
extern crate test;

mod quiz;
mod dispatcher;
mod utils;
mod top;
mod antimoon;
mod captcha;
pub(crate) mod markdown;
pub(crate) mod users;

use std::env;
use std::error::Error;
use telegram_bot::Api;
use tokio::sync::Mutex;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let token = env::var("TELEGRAM_BOT_TOKEN");
    let users = Arc::new(Mutex::new(users::Users::new(env::var("USERS_DB").expect("USERS_DB not set")).unwrap()));
    let api = Api::new(token.unwrap());

    let mut disp = dispatcher::Dispatcher::new(api.clone());

    let quiz = quiz::QuizModule::new(api.clone(), users.clone());
    disp.add_sub("quiz".to_string(), &quiz);

    let top = top::UserTopModule::new(api.clone(), users.clone());
    disp.add_sub("top".to_string(), &top);
    
    let antimoon = antimoon::Antimoon::new(api.clone());
    disp.add_sub("antimoon".to_string(), &antimoon);
    
    let captcha = captcha::Captcha::new(api.clone());
    disp.add_sub("captcha".to_string(), &captcha);

    disp.start().await.unwrap();
    Ok(())
}
