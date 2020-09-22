use crate::quiz::{Poll};
use telegram_bot::{UserId};
use crate::markdown;

fn user_list(data: &Vec<(UserId, String)>) -> String {
    let mut f = false;
    let mut text = String::new();
    for i in data {
        if f { text += ", " }
        text += &*markdown::bold(&*markdown::escape(&i.1));
        f = true;
    }
    text
}

pub(crate) fn poll_result<'s>(poll: &Poll) -> String {
    if poll.correct_answers.len() == 0 && poll.incorrect_answers.len() == 0 {
        markdown::escape("Никто не ответил:с
        Ну и сами себе вопросы загадывайте!")
    } else {
        let mut text = String::new();
        if poll.correct_answers.len() > 0 {
            text += "Список Кодзим: ";
            text += &user_list(&poll.correct_answers);
        };
        if poll.incorrect_answers.len() > 0 {
            text += "\nСписок дэбилов: ";
            text += &user_list(&poll.incorrect_answers);
        };
        text
    }
}