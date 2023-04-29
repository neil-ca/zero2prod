use secrecy::ExposeSecret;
use sqlx::PgPool;
use zero2prod::telemetry::{get_subscriber, init_subscriber};
use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;
#[tokio::main]
async fn main() -> std::io::Result<()> {
    // The `with` method is povided by `SubscriberExt`, an extension
    // trait for `Subscriber` exposed by `tracing_subscriber`
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // Panic if we can't read configuration
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_pool = PgPool::connect(&configuration.database.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await?;
    Ok(())
}

//async fn greet(req: HttpRequest) -> impl Responder {
//let name = req.match_info().get("name").unwrap_or("World");
//format!("Hello {}!", &name)
//}
