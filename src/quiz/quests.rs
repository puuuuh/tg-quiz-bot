use rusqlite::{Connection, Error};
use std::path::Path;
use fallible_iterator::FallibleIterator;
use rand::prelude::SliceRandom;
use rusqlite::params;
use rand::thread_rng;

#[derive(Debug)]
pub enum QuesterError {
    DBError(rusqlite::Error)
}

impl From<rusqlite::Error> for QuesterError {
    fn from(e: Error) -> Self {
        QuesterError::DBError(e)
    }
}

#[derive(Debug)]
pub struct Question {
    pub text: String,
    pub answers: Vec<(String, bool)>,
}

pub struct Quester {
    db: Connection,
}

impl Quester {
    pub fn new<T: AsRef<Path>>(path: T) -> Result<Quester, rusqlite::Error> {
        Ok(Quester{
            db: Connection::open(path)?,
        })
    }

    pub fn get_quest(&self) -> Result<Question, QuesterError> {
        let mut select_quest = self.db.prepare(
            "SELECT * FROM questions ORDER BY RANDOM() LIMIT 1;",
        )?;
        let mut select_answers = self.db.prepare(
            "SELECT answer,valid FROM answers WHERE question_id=?;",
        )?;

        let mut test = select_quest.query(params![])?;
        if let Some(res) = test.next()? {
            let id = res.get::<usize, i32>(0)?;
            let quest = res.get::<usize, String>(1)?;
            let mut answers = select_answers.query(params![id])?.map(|row| {
                Ok((row.get::<usize, String>(0)?, row.get::<usize, i32>(1)? == 1))
            }).collect::<Vec<(String, bool)>>()?;
            answers.shuffle(&mut thread_rng());
            Ok(Question {
                text: quest,
                answers
            })
        } else {
            unreachable!()
        }
    }
}