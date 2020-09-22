use rusqlite::{Connection, Error};
use std::path::Path;
use fallible_iterator::FallibleIterator;
use rusqlite::params;

#[derive(Debug)]
pub enum UsersError {
    DBError(rusqlite::Error)
}

impl From<rusqlite::Error> for UsersError {
    fn from(e: Error) -> Self {
        UsersError::DBError(e)
    }
}

#[derive(Debug)]
pub struct User {
    pub uid: i64,
    pub first_name: String,
    pub last_name: String,
    pub username: String
}

pub struct Users {
    db: Connection,
}

impl Users {
    pub fn new<T: AsRef<Path>>(path: T) -> Result<Users, rusqlite::Error> {
        let db = Connection::open(path)?;
        db.execute("CREATE TABLE IF NOT EXISTS users
                        (uid INTEGER UNIQUE PRIMARY KEY, first_name TEXT, last_name TEXT, username TEXT)", params![])?;
        db.execute("CREATE UNIQUE INDEX IF NOT EXISTS id_index ON users (uid)", params![])?;
        db.execute("CREATE TABLE IF NOT EXISTS scores
                        (uid INTEGER UNIQUE PRIMARY KEY, score INTEGER)", params![])?;
        db.execute("CREATE UNIQUE INDEX IF NOT EXISTS id_index ON scores (uid)", params![])?;
        Ok(Users{
            db,
        })
    }

    pub fn get_rating(&mut self, uid: i64) -> Result<i64, UsersError> {
        let mut select_quest = self.db.prepare(
            "SELECT rating FROM users WHERE user_id=?",
        )?;

        let mut test = select_quest.query(params![uid])?;
        if let Some(res) = test.next()? {
            Ok(res.get(0)?)
        } else {
            Ok(0)
        }
    }

    pub fn get_top(&mut self, count: i64) -> Result<Vec<(User, i64)>, UsersError> {
        let mut select_users = self.db.prepare(
            "SELECT users.uid, users.first_name, users.last_name, users.username, score FROM scores JOIN users on users.uid = scores.uid ORDER BY score DESC LIMIT ?",
        )?;

        let users = select_users.query(params![count])?.map(|row| {
            let user = User {
                uid: row.get::<usize, i64>(0).unwrap(),
                first_name: row.get::<usize, String>(1).unwrap(),
                last_name: row.get::<usize, String>(2).unwrap(),
                username: row.get::<usize, String>(3).unwrap(),
            };

            Ok( (user, row.get::<usize, i64>(4).unwrap()) )
        }).collect::<Vec<(User, i64)>>()?;
        Ok(users)
    }

    pub fn update_user(&mut self, user: &User) -> Result<(), UsersError> {
        let mut update = self.db.prepare(
            "INSERT INTO users (uid, first_name, last_name, username)
                     VALUES(?, ?, ?, ?)
                     ON CONFLICT(uid)
                     DO UPDATE SET first_name = ?, last_name = ?, username = ?",
        )?;

        update.execute(params![user.uid, &user.first_name, &user.last_name, &user.username, &user.first_name, &user.last_name, &user.username])?;
        Ok(())
    }

    pub fn inc_rating(&mut self, uid: i64, rating: i64) -> Result<(), UsersError> {
        let mut select_quest = self.db.prepare(
            "INSERT INTO scores (uid, score)
                     VALUES(?, ?)
                     ON CONFLICT(uid)
                     DO UPDATE SET score = score + ?",
        )?;

        select_quest.execute(params![uid, rating, rating])?;
        Ok(())
    }
}