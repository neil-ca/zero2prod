use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpResponse, HttpServer};

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

//pub async fn run() -> std::io::Result<()> {
//HttpServer::new(|| {
//App::new()
//.route("/health_check", web::get().to(health_check))
//.route("/", web::get().to(index))
//.route("/{name}", web::get().to(greet))
//})
//.bind("127.0.0.1:8000")?
//.run()
//.await
//}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| App::new().route("health_check", web::get().to(health_check)))
        .listen(listener)?
        .run();

    Ok(server)
}
