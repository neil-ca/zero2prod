use std::{write, writeln};

use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::EmailClient,
    startup::ApplicationBaseUrl,
};
use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;
// An extension trait to provide the `graphemes` method
// on `String` and `&str`
use actix_web::http::StatusCode;
use actix_web::ResponseError;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use unicode_segmentation::UnicodeSegmentation;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;
    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(NewSubscriber { email, name })
    }
}
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool, email_client, base_url),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
) -> Result<HttpResponse, SubscribeError> {
    let new_subscriber = form.0.try_into().map_err(SubscribeError::ValidationError)?;
    let mut transaction = pool.begin().await.map_err(SubscribeError::PoolError)?;
    let subscriber_id = insert_subscriber(&mut transaction, &new_subscriber)
        .await
        .map_err(SubscribeError::TransactionCommitError)?;
    let subscription_token = generate_subscription_token();
    store_token(&mut transaction, subscriber_id, &subscription_token).await?;

    transaction.commit().await.map_err(SubscribeError::TransactionCommitError)?;
    send_confirmation_email(
        &email_client,
        new_subscriber,
        &base_url.0,
        &subscription_token,
    )
    .await?;
    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber, base_url, subscription_token)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
    subscription_token: &str,
) -> Result<(), reqwest::Error> {
    // Send a (useless) email to the new subscriber
    // We are ignoring email delivery errors for now
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, subscription_token
    );
    let plain_body = &format!(
        "Welcome to oue newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );
    let html_body = &format!(
        "Welcome to our newsletter!<br/>\
        Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );
    email_client
        .send_email(new_subscriber.email, "Welcome", &html_body, &plain_body)
        .await
}

pub fn is_valid_name(s: &str) -> bool {
    let is_empty_or_whitespace = s.trim().is_empty();
    let is_too_long = s.graphemes(true).count() > 256;

    let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
    let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));
    !(is_empty_or_whitespace || is_too_long || contains_forbidden_characters)
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, transaction)
)]
pub async fn insert_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    new_subscriber: &NewSubscriber,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at, status)
    VALUES ($1, $2, $3, $4, 'pending_confirmation')
            "#,
        subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(transaction)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(subscriber_id)
}

// Generate a random 25-characters-long case-sensitive subscription token
fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

#[tracing::instrument(
    name = "Store subscription token in the database",
    skip(subscription_token, transaction)
)]
pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), StoreTokenError> {
    sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscription_token, subscription_id) 
        VALUES ($1, $2)"#,
        subscription_token,
        subscriber_id
    )
    .execute(transaction)
    .await
    .map_err(|e| StoreTokenError(e))?;
    Ok(())
}

pub struct StoreTokenError(sqlx::Error);

impl std::fmt::Debug for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}
impl std::fmt::Display for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A database error was encountered while \
               trying to store a subscription token."
        )
    }
}
impl ResponseError for StoreTokenError {}

impl std::error::Error for StoreTokenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // The compiler transparently casts `&sqlx::Error` into a `&dyn Error`
        Some(&self.0)
    }
}

fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}

#[derive(thiserror::Error)]
pub enum SubscribeError {
    #[error("{0}")]
    ValidationError(String),
    #[error("Failed to store the confirmation token for a new subscriber")]
    StoreTokenError(#[from] StoreTokenError),
    #[error("Failed to send a confirmation email")]
    SendEmailError(#[from] reqwest::Error),
    #[error("Failed to acquire a Postgres connection from the pool")]
    PoolError(#[source] sqlx::Error),
    #[error("Failed to insert new subscriber in the database")]
    InsertSubscriberError(#[source] sqlx::Error),
    #[error("Failed to commit SQL transaction to store a new subscriber")]
    TransactionCommitError(#[source] sqlx::Error)
}

impl ResponseError for SubscribeError {
    fn status_code(&self) -> StatusCode {
        match self {
            SubscribeError::ValidationError(_) => StatusCode::BAD_REQUEST,
            SubscribeError::PoolError(_)
            | SubscribeError::TransactionCommitError(_)
            | SubscribeError::InsertSubscriberError(_)
            | SubscribeError::StoreTokenError(_)
            | SubscribeError::SendEmailError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl std::fmt::Debug for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

// Without thiserror 
//
// impl std::error::Error for SubscribeError {
//     fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
//         match self {
//             SubscribeError::ValidationError(_) => None,
//             SubscribeError::StoreTokenError(e) => Some(e),
//             SubscribeError::SendEmailError(e) => Some(e),
//             SubscribeError::PoolError(e) => Some(e),
//             SubscribeError::InsertSubscriberError(e) => Some(e),
//             SubscribeError::TransactionCommitError(e) => Some(e)
//         }
//     }
// }
//
//
// impl std::fmt::Display for SubscribeError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             SubscribeError::ValidationError(e) => write!(f, "{}", e),
//             SubscribeError::StoreTokenError(_) => write!(
//                 f,
//                 "Failed to store the confirmation token for a new subscriber"
//             ),
//             SubscribeError::SendEmailError(_) => {
//                 write!(f, "Failed to send a confirmation email.")
//             }
//             SubscribeError::PoolError(_) => {
//                 write!(f, "Failed to acquire a Postgres connection from the pool")
//             }
//             SubscribeError::InsertSubscriberError(_) => {
//                 write!(f, "Failed to insert new subscriber in the database")
//             }
//             SubscribeError::TransactionCommitError(_) => {
//                 write!(f, "Failed to commit SQL transaction to store a new subscriber")
//             }
//         }
//     }
// }
//
// impl From<reqwest::Error> for SubscribeError {
//     fn from(e: reqwest::Error) -> Self {
//         Self::SendEmailError(e)
//     }
// }
//
// impl From<StoreTokenError> for SubscribeError {
//     fn from(e: StoreTokenError) -> Self {
//         Self::StoreTokenError(e)
//     }
// }
//
// impl From<String> for SubscribeError {
//     fn from(e: String) -> Self {
//         Self::ValidationError(e)
//     }
// }
