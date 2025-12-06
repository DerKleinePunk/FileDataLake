use actix_files as fs;

//https://github.com/actix/examples
use actix_web::{get, web, App, HttpServer, Responder, middleware::Logger};

#[get("/api/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {}!", name)
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    //env::set_var("RUST_LOG", "actix_web=info");
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    println!("starting web server on 127.0.0.1:8080");

    HttpServer::new(|| {
        App::new()
            // Add the Logger middleware to log all incoming requests.
            .wrap(Logger::new("%a %{User-Agent}i"))
            .service(greet)
            .service(fs::Files::new("", "./wwwroot")
                .use_last_modified(true)
                .use_etag(true)
                .prefer_utf8(true)
                .index_file("index.html"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
