use actix_files as acfs;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, middleware::Logger, post, web::{self, Data}};
use actix_multipart::{form::{MultipartForm, tempfile::TempFile}};
use std::{env, path::PathBuf, sync::Mutex, fs};

//mod files;

//https://github.com/actix/examples
//https://github.com/actix/examples/blob/master/databases/sqlite/src/main.rs

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
    data: Data<Mutex<MyAppData>>,
     MultipartForm(form): MultipartForm<Upload>
) -> impl Responder {

     let app_data = data.lock().unwrap();

    for f in form.files {
        let temp_file_path = f.file.path();
        let mut file_path = PathBuf::from(&app_data.upload_path);
        file_path = file_path.join(f.file_name.unwrap());
        log::debug!("copy file from {temp_file_path:?} to {file_path:?}");
        match std::fs::copy(temp_file_path, file_path) {
            Ok(_) => {log::debug!("done")}
            Err(error) =>{
                log::error!("{error:?}");
                return HttpResponse::InternalServerError().body(error.to_string())
            }
        }
    }
    HttpResponse::Ok().finish()
}

struct MyAppData {
    upload_path: String,
}

//https://github.com/deadpool-rs/deadpool/blob/main/examples/redis-actix-web/src/main.rs

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    //Todo add to config file
    let mut my_app_data = MyAppData { upload_path: "./upload".to_string() };

    let path = env::current_dir()?;
    println!("The current directory is {}", path.display());

    let upload_dir = PathBuf::from(my_app_data.upload_path);
    my_app_data.upload_path = fs::canonicalize(&upload_dir)?.to_string_lossy().to_string();

    let data = Data::new(Mutex::new(my_app_data));

    println!("starting web server on 127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            // Add the Logger middleware to log all incoming requests.
            .wrap(Logger::new("%a %{User-Agent}i"))
            .app_data(Data::clone(&data))
            .service(greet)
            .service(save_file_server)
            .service(acfs::Files::new("", "./wwwroot")
                .use_last_modified(true)
                .use_etag(true)
                .prefer_utf8(true)
                .index_file("index.html"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
