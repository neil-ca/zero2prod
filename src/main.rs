use std::net::TcpListener;
use env_logger::Env;
use sqlx::PgPool;
use zero2prod::startup::run;
use zero2prod::configuration::get_configuration;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    
    // `init` does call `set_logger`, so this is all we need to do.
    // We are fallingback to printing all logs at info-level or above
    // if the RUST_LOG environment variable has not been set
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Panic if we can't read configuration
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
            .await
            .expect("Failed to connect to Postgres");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await
}

//async fn greet(req: HttpRequest) -> impl Responder {
//let name = req.match_info().get("name").unwrap_or("World");
//format!("Hello {}!", &name)
//}
