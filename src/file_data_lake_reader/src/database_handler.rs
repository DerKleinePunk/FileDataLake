use std::path::Path;
use rusqlite::{Connection, Statement, Result};
use deadpool_sqlite::{Config, Runtime};

//tokio = { version = "1.12.0", features = ["rt", "rt-multi-thread", "macros"] }
use crate::app_dtos::FileEntry;

#[derive(Debug)]
pub struct LocalDbState {
    conn_pool: deadpool_sqlite::Pool,
    checked: bool
}

impl LocalDbState {
    pub fn new(file_name: &Path) -> LocalDbState {
        let cfg = Config::new(file_name);
        let pool = cfg.create_pool(Runtime::Tokio1).unwrap();
        //let conn = Connection::open(file_name).unwrap();
        LocalDbState { conn_pool: pool, checked: false}
    }

    pub fn get(self) -> deadpool_sqlite::Pool {
        return self.conn_pool;
    }

    pub async fn create_database(dbstate : &mut LocalDbState) -> Result<(),rusqlite::Error> {
        let conn = dbstate.conn_pool.get().await.unwrap();
        let _ = conn.interact(|conn| {
            let result = conn.execute(
                "create table if not exists files (
                    id BLOB PRIMARY KEY,
                    name_org text not null,
                    hash text not null,
                    size integer not null
                )",
                (),
            );
            if result.is_err(){
                return;
            }
            _= conn.execute(
            "create table if not exists file_attributes (
                id_file BLOB not null references files(id),
                name text not null,
                value text not null,
                PRIMARY KEY(id_file,name)
            )",
            (),
            )
        }).await;
        dbstate.checked = true;

        Ok(())
    }

    pub async fn save_file_info(conn_pool : deadpool_sqlite::Pool, file_entry: &FileEntry) -> Result<()> {
        let conn = conn_pool.get().await.unwrap();

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
