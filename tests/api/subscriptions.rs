use crate::helpers::spawn_app; 
#[tokio::test]
async fn subscribe_resturns_a_200_for_valid_form_data() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=neil%20cam&email=neil%40gmail.com";

    //for (body, description) in test_cases {

    let response = app.post_subscribtion(body.into()).await;
    assert_eq!(
        200,
        response.status().as_u16(),
    );
    //}
    // Act

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
    assert_eq!(saved.name, "neil cam");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let app = spawn_app().await;
    let test_cases = vec![
        ("name=neil%20ulises", "missing the email"),
        ("email=neil%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = app.post_subscribtion(invalid_body.into()).await;
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        )
    }
}

#[tokio::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_invalid() {
    // Arrange
    let app = spawn_app().await;
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];

    for (body, description) in test_cases {
        // Act
        let response = app.post_subscribtion(body.into()).await;
        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 Bad Request when the payload was {}.",
            description
        );
    }
}
