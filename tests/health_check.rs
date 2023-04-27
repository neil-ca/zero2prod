use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::startup::run;
use sqlx::{PgPool, PgConnection, Connection, Executor};
use zero2prod::configuration::{get_configuration, DatabaseSettings};

#[derive(Debug)]
pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}
// No .await call, therefore no need for `spawn_app` to be async now.
// We are also running tests, so it is not worth it to propagate errors:
// if we fail to perform the required setup we can just panic and crash 
// all the things.
// The fn is async now
async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind random port");
    // We retrieve the port assigned to us by the OS 
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("Failed to read config");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let connection_pool = configure_database(&configuration.database)
        .await;

    let server = run(listener, connection_pool.clone()).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    TestApp {
        address,
        db_pool: connection_pool
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database 
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
       .expect("Failed to create database");

    // Migrate database 
    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to create database");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate database");
    connection_pool
}
#[tokio::test] 
async fn healt_check_works() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client.get(&format!("{}/health_check", &address.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_resturns_a_200_for_valid_form_data() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let body = "name=neil%20ulises&email=neil%40gmail.com";
    // Act 
    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    //let configuration = get_configuration().expect("Failed to read configuration");
    //let connection_string = configuration.database.connection_string();
    // The `Connection` trait MUST be in scope for us to invoke
    // `PgConnection::connect` - it is not an inherent method of the struct!
    //let mut connection = PgConnection::connect(&connection_string)
        //.await
        //.expect("Failed to connect to Postgres");

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved sub");
    assert_eq!(saved.email, "neil@gmail.com");
    assert_eq!(saved.name, "neil ulises");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=neil%20ulises", "missing the email"),
        ("email=neil%40gmail.com", "missing the name"),
        ("", "missing both name and email")
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");
        assert_eq!(400, response.status().as_u16(), "The API did not fail with 400 Bad Request when the payload was {}.", error_message)
    }
}
