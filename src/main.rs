use actix_web::{web, App, HttpResponse, HttpServer, Responder, HttpRequest};

//async fn greet() -> String {
    //format!("Welcome !")
//}

async fn index() -> HttpResponse {
    HttpResponse::Ok().body("Hello!")
}

async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}
async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}
#[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/health_check", web::get().to(health_check))
            .route("/{name}", web::get().to(greet))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
