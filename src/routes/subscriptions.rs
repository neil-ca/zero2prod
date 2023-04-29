use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    let request_id = Uuid::new_v4();
    // spans, like logs, have an assoiciated level
    // 'info_span' creates a span at the info-level
    let request_span = tracing::info_span!(
    "Adding a new subscriber.",
    %request_id,
    subscriber_email = %form.email,
    subscribe_name = %form.name
                                      );

    // Using `enter` in an async function is a recipe four disaster!
    let _request_span_guard = request_span.enter();
    tracing::info!(
        "request_id '{}' - Adding '{}' '{}' as a new subscriber",
        request_id,
        form.email,
        form.name
    );
    tracing::info!(
        "request_id {} - Sabing new subscriber details in the database",
        request_id
    );
    match sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES ($1, $2, $3, $4)
            "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(pool.as_ref())
    .await
    {
        Ok(_) => {
            tracing::info!(
                "request_id {} - New subscriber details have been saved",
                request_id
            );
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            tracing::error!(
                "request_id {} - Failed to execute query: {:?}",
                request_id,
                e
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}
