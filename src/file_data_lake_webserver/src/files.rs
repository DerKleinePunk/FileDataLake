use std::io::Write;
use actix_multipart::Multipart;
use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer};
use futures::{StreamExt, TryStreamExt};

    /*let upload_status = files::save_file(payload, "/path/filename.jpg".to_string()).await;

    match upload_status {
        Some(true) => {

            HttpResponse::Ok().body(format!("File Upload Ok"))
        }
        _ => HttpResponse::InternalServerError()
            .body("File Upload failed"),
    }*/
    
pub async fn save_file(mut payload: Multipart, file_path: String) -> Option<bool> {
    // iterate over multipart stream
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition().unwrap();
        //let filename = content_type.get_filename().unwrap();
        let filepath = format!(".{}", file_path);

        // File::create is blocking operation, use threadpool
        let mut f = web::block(|| std::fs::File::create(filepath))
            .await
            .unwrap().unwrap();

        // Field in turn is stream of *Bytes* object
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            // filesystem operations are blocking, we have to use threadpool
            f = web::block(move || f.write_all(&data).map(|_| f))
                .await
                .unwrap().unwrap();
        }
    }

    Some(true)
}
