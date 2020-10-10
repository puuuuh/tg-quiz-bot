use rusqlite::*;

#[derive(Debug)]
pub enum ModulesError {
    DBError(rusqlite::Error),
}

impl From<rusqlite::Error> for ModulesError {
    fn from(e: Error) -> Self {
        ModulesError::DBError(e)
    }
}

pub(crate) struct Modules {
    conn: rusqlite::Connection,
}

impl Modules {
    pub fn new<T: AsRef<std::path::Path>>(path: T) -> Result<Modules, ModulesError> {
        let db = Connection::open(path)?;
        db.execute(
            "CREATE TABLE IF NOT EXISTS modules
                        (chat_id INTEGER, name TEXT,
                        PRIMARY KEY(chat_id, name))",
            params![],
        )?;
        Ok(Modules { conn: db })
    }

    pub fn modules(&mut self) ->Result<Vec<(i64, String)>, ModulesError> {
        let mut query = self.conn.prepare("SELECT * FROM modules")?;
        let res = query.query_map(NO_PARAMS, |x| {
            Ok((x.get(0)?, x.get(1)?)) 
        })?.map(|x| x.unwrap()).collect();

        Ok(res)
    }
    
    pub fn add(&mut self, chat: i64, name: &str) -> Result<(), ModulesError> {
        let mut query = self.conn.prepare("INSERT INTO modules (chat_id, name) VALUES(?, ?)")?;
        query.execute(params![chat, name])?;
        Ok(())
    }
    pub fn rm(&mut self, chat: i64, name: &str) -> Result<(), ModulesError> {
        let mut query = self.conn.prepare("DELETE FROM modules WHERE chat_id=? AND name=?")?;
        query.execute(params![chat, name])?;
        Ok(())

    }
}
