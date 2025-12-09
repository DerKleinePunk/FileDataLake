use actix_files as acfs;
use actix_multipart::form::{MultipartForm, tempfile::TempFile};
use actix_web::{
    App, HttpResponse, HttpServer, Responder, get,
    middleware::Logger,
    post,
    web::{self, Data},
};
use deadpool_sqlite::{
    Config, Pool, PoolError, Runtime,
    rusqlite::{Row, params},
};
use serde::{Deserialize, Serialize};
use std::io::Error;
use std::{env, fs, io::ErrorKind, path::PathBuf};

//mod files;

//https://github.com/actix/examples
//https://github.com/actix/examples/blob/master/databases/sqlite/src/main.rs
//https://docs.rs/rusqlite/latest/rusqlite/struct.Statement.html

#[get("/api/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {}!", name)
}

#[derive(Debug, MultipartForm)]
pub struct Upload {
    #[multipart(rename = "file")]
    files: Vec<TempFile>,
}

#[post("/api/file/save")]
async fn save_file_server(
    data: web::Data<MyAppData>,
    MultipartForm(form): MultipartForm<Upload>,
) -> impl Responder {
    for f in form.files {
        let temp_file_path = f.file.path();
        let mut file_path = PathBuf::from(&data.upload_path);
        file_path = file_path.join(f.file_name.unwrap());
        log::debug!("copy file from {temp_file_path:?} to {file_path:?}");
        match std::fs::copy(temp_file_path, file_path) {
            Ok(_) => {
                log::debug!("done")
            }
            Err(error) => {
                log::error!("{error:?}");
                return HttpResponse::InternalServerError().body(error.to_string());
            }
        }
    }

    return HttpResponse::SeeOther()
        .insert_header(("Location", "/"))
        .finish();
}

#[derive(Debug, Deserialize)]
struct WhereRequest {
    field: Option<String>,
    value: Option<String>,
}

#[derive(Debug, Serialize)]
struct FileCountResponse {
    files: u64,
}

fn get_result(row: &Row) -> Result<u64, deadpool_sqlite::rusqlite::Error> {
    return row.get(0);
}

async fn file_count(
    pool: &Pool,
    where_request: WhereRequest,
) -> Result<FileCountResponse, PoolError> {
    log::debug!("we get info {where_request:?}");

    let mut sql_text = "select count(*) from files".to_string();

    if where_request.field.is_some() && where_request.value.is_some() {
        //Little Sql Inject check no spaces
        let field_name = where_request.field.unwrap().replace(" ", "").replace(";", "");
        sql_text = format!("select count(*) from files where {} like ?", field_name);
    }

    let mut value = "".to_string();
    let mut use_parameter = false;
    if where_request.value.is_some() {
        value = where_request.value.unwrap();
        use_parameter = true;
    }
    let conn = pool.get().await.unwrap();
    let sql_result = conn
        .interact(move |conn| {
            let result : Result<u64, deadpool_sqlite::rusqlite::Error>;
            if use_parameter {
                let params = params![value];
                result = conn.query_one(sql_text.as_str(), params, get_result);
            }
            else {
                result = conn.query_one(sql_text.as_str(), params![], get_result);
            }

            if result.is_err() {
                let sql_error = result.err().unwrap();
                log::error!("{sql_error:?}");
                return 0;
            }
            return result.unwrap();
        })
        .await
        .unwrap();

    let response = FileCountResponse { files: sql_result };
    Ok(response)
}

#[get("/api/file/count")]
async fn get_file_cont(
    data: web::Data<MyAppData>,
    info: web::Query<WhereRequest>,
) -> impl Responder {
    log::debug!("we get info {info:?}");

    let response = file_count(&data.conn_pool, info.into_inner())
        .await
        .unwrap();

    /*let mut response = FileCountResponse { files: 0 };

    if info.field == Some("test".to_string()) {
        response.files = 1;
        if info.value == Some("value".to_string()) {
            response.files = 2;
        }
    }*/

    HttpResponse::Ok().json(response)
}

struct MyAppData {
    upload_path: String,
    conn_pool: deadpool_sqlite::Pool,
}

/// `AppConfigFile` implements `Default`
impl ::std::default::Default for AppConfigFile {
    fn default() -> Self {
        Self {
            version: 0,
            database: "./target/fdl.db3".into(),
            upload_path: "./upload".into(),
            bind_ip: "127.0.0.1".into(),
            port: 8080,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct AppConfigFile {
    version: u64,
    database: String,
    upload_path: String,
    bind_ip: String,
    port: u16,
}

//https://github.com/deadpool-rs/deadpool/blob/main/examples/redis-actix-web/src/main.rs

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    let config_path = confy::get_configuration_file_path("fdl", "web");
    match config_path {
        Ok(result) => log::debug!("Config is hier {result:?}"),
        Err(error) => {
            eprintln!("Error: {error:?}");
            let error = Error::new(ErrorKind::InvalidData, error);
            return Err(error);
        }
    }

    let config_file: AppConfigFile = confy::load("fdl", "web").unwrap();

    //Save so User see the new Defaults
    confy::store("fdl", "web", &config_file).unwrap();

    //Todo add to config file
    let cfg = Config::new(config_file.database);

    let mut my_app_data = MyAppData {
        upload_path: config_file.upload_path.clone(),
        conn_pool: cfg.create_pool(Runtime::Tokio1).unwrap(),
    };

    let path = env::current_dir()?;
    println!("The current directory is {}", path.display());

    let upload_dir = PathBuf::from(my_app_data.upload_path);

    my_app_data.upload_path = fs::canonicalize(&upload_dir)?.to_string_lossy().to_string();

    let up_load_path = my_app_data.upload_path.to_string();
    let data = Data::new(my_app_data);

    println!("starting web server on 127.0.0.1:8080");
    println!("Upload file are {up_load_path:?}");

    HttpServer::new(move || {
        App::new()
            // Add the Logger middleware to log all incoming requests.
            .wrap(Logger::new("%a %{User-Agent}i"))
            .app_data(Data::clone(&data))
            .service(greet)
            .service(save_file_server)
            .service(get_file_cont)
            .service(
                acfs::Files::new("", "./wwwroot")
                    .use_last_modified(true)
                    .use_etag(true)
                    .prefer_utf8(true)
                    .index_file("index.html"),
            )
    })
    .workers(4)
    .bind((config_file.bind_ip, config_file.port))?
    .run()
    .await
}
