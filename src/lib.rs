use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpResponse, HttpServer};

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String
}

async fn subscribe(_form: web::Form<FormData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
//pub enum Payload<S = Pin<Box<dyn Stream<Item = Result<Bytes, PayloadError>> + 'static, Global>>> {
    //None,
    //H1 {
        //payload: Payload,
    //},
    //H2 {
        //payload: Payload,
    //},
    //Stream {
        //payload: S,
    //},
//}
