use actix_web::{web, HttpServer, App };

async fn greet() -> String {
    format!("Welcome !")
}


#[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(greet))
            .route("/", web::get().to(greet))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}

