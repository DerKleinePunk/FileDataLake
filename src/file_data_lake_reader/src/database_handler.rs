use std::path::Path;
use rusqlite::{Connection, Statement, Result};

use crate::app_dtos::FileEntry;

#[derive(Debug)]
pub struct LocalDbState {
    conn: Connection,
    checked: bool
}

impl LocalDbState {
    pub fn new(file_name: &Path) -> LocalDbState {
        let conn = Connection::open(file_name).unwrap();
        LocalDbState { conn: conn, checked: false}
    }

    pub fn create_database(dbstate : &mut LocalDbState) -> Result<()> {
         dbstate.conn.execute(
            "create table if not exists files (
                id BLOB PRIMARY KEY,
                name_org text not null,
                hash text not null,
                size integer not null
            )",
            (),
        )?;
        dbstate.conn.execute(
            "create table if not exists file_attributes (
                id_file BLOB not null references files(id),
                name text not null,
                value text not null,
                PRIMARY KEY(id_file,name)
            )",
            (),
        )?;
        dbstate.checked = true;

        Ok(())
    }

    pub fn save_file_info(self, file_entry: &FileEntry) -> Result<()> {

        self.conn.prepare(sql).unwrap();

        Ok(())
    }
}

struct PreparedStatement<'conn> {
    statement: Statement<'conn>,
}

impl<'conn> PreparedStatement<'conn> {
    pub fn new<'a>(conn: &'a Connection, sql: &str) -> PreparedStatement<'a> {
        PreparedStatement {
            statement: conn.prepare(sql).unwrap(),
        }
    }

    fn query_some_info(&mut self, arg: i64) -> Result<i64, rusqlite::Error> {
        let mut result_iter = self.statement.query(&[&arg]).unwrap();
        let result = result_iter.next().unwrap().unwrap().get(0);

        result
    }
}
