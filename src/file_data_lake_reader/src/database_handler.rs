use deadpool_sqlite::{Config, InteractError, Runtime};
use rusqlite::{Connection, Result, Statement, named_params};
use std::path::Path;

//tokio = { version = "1.12.0", features = ["rt", "rt-multi-thread", "macros"] }
use crate::app_dtos::FileEntry;

#[derive(Debug)]
pub struct LocalDbState {
    conn_pool: deadpool_sqlite::Pool,
    checked: bool,
}

impl LocalDbState {
    pub fn new(file_name: &Path) -> LocalDbState {
        let cfg = Config::new(file_name);
        let pool = cfg.create_pool(Runtime::Tokio1).unwrap();
        //let conn = Connection::open(file_name).unwrap();
        LocalDbState {
            conn_pool: pool,
            checked: false,
        }
    }

    pub fn get(self) -> deadpool_sqlite::Pool {
        return self.conn_pool;
    }

    pub async fn create_database(dbstate: &mut LocalDbState) -> Result<(), rusqlite::Error> {
        let conn = dbstate.conn_pool.get().await.unwrap();
        let _ = conn
            .interact(|conn| {
                let result = conn.execute(
                    "create table if not exists files (
                    id BLOB PRIMARY KEY,
                    name_org text not null,
                    hash text not null,
                    size integer not null
                )",
                    (),
                );
                if result.is_err() {
                    return;
                }
                _ = conn.execute(
                    "create table if not exists file_attributes (
                id_file BLOB not null references files(id),
                name text not null,
                value text not null,
                PRIMARY KEY(id_file,name)
            )",
                    (),
                )
            })
            .await;
        dbstate.checked = true;

        Ok(())
    }

    pub async fn save_file_info(
        conn_pool: deadpool_sqlite::Pool,
        file_entry: FileEntry,
    ) -> Result<(), rusqlite::Error> {
        let conn = conn_pool.get().await.unwrap();
          let sql_result_future = conn.interact(move |conn| {
            let mut statement = conn.prepare("insert into files (id, name_org, hash, size) values (:id, :name, :hash, :size)").unwrap();
            let sql_result = statement.execute(named_params! {
                ":id": file_entry.id.as_bytes(),
                ":name": file_entry.name,
                ":hash": file_entry.hash,
                ":size": file_entry.size});
            if sql_result.is_err() {
                let sql_error = sql_result.err().unwrap();
                log::error!("sql_result {sql_error:?}");
                return;
            }
        }).await;
        //todo how to get the error
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
