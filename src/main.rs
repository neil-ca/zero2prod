use std::net::TcpListener;

use zero2prod::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Bubble up the io::Error if we failed to bind the address
    // otherwise call .await on our Server
    let listener = TcpListener::bind("127.0.0.1:8000").expect("Failed to bind random port");

    run(listener)?.await
}

//async fn greet() -> String {
//format!("Welcome !")
//}

//async fn index() -> HttpResponse {
//HttpResponse::Ok().body("Hello!")
//}

//async fn greet(req: HttpRequest) -> impl Responder {
//let name = req.match_info().get("name").unwrap_or("World");
//format!("Hello {}!", &name)
//}
