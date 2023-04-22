use std::net::TcpListener;
use zero2prod::startup::run;
use zero2prod::configuration::get_configuration;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Panic if we can't read configuration
    let configuration = get_configuration().expect("Failed to read configuration");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener)?.await
}

//async fn greet(req: HttpRequest) -> impl Responder {
//let name = req.match_info().get("name").unwrap_or("World");
//format!("Hello {}!", &name)
//}
