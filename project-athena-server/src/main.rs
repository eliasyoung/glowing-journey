#![allow(unused)]

use std::net::ToSocketAddrs;

use tracing::Level;

use argon2::{password_hash::SaltString, Argon2, PasswordHash};
use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{filter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Hello, world!");

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            filter::Targets::new()
                .with_target("tower_http::trace::on_response", Level::TRACE)
                .with_target("tower_http::trace::on_request", Level::TRACE)
                .with_target("tower_http::trace::make_span", Level::DEBUG)
                .with_default(Level::INFO),
        )
        .init();

    let db = PgPoolOptions::new()
        // The default connection limit for a Postgres server is 100 connections, minus 3 for superusers.
        // Since we're using the default superuser we don't have to worry about this too much,
        // although we should leave some connections available for manual access.
        //
        // If you're deploying your application with multiple replicas, then the total
        // across all replicas should not exceed the Postgres connection limit.
        .max_connections(50)
        .connect("postgresql://admin:secret@localhost/athena_db")
        .await?;

    tracing::info!("DB connect successfully!");

    sqlx::migrate!("db/migrations/").run(&db).await?;

    let app = Router::new()
        .route("/", get(root))
        .route("/users", post(create_user).get(get_user))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    let addr = listener.local_addr()?.to_string();

    axum::serve(listener, app).await.unwrap();

    async fn root() -> &'static str {
        "Hello, World!"
    }

    async fn create_user(Json(payload): Json<CreateUser>) -> (StatusCode, Json<User>) {
        // insert your application logic here

        let user = User {
            username: payload.username,
            password: payload.password,
            email: payload.email,
        };

        (StatusCode::CREATED, Json(user))
    }

    async fn get_user() -> (StatusCode, Json<User>) {
        let user = User {
            username: String::from("Kobe Bryant"),
            email: String::from("Kobe Bryant"),
            password: String::from("Kobe Bryant"),
        };

        (StatusCode::OK, Json(user))
    }

    #[derive(Deserialize)]
    struct CreateUser {
        username: String,
        email: String,
        password: String,
    }

    #[derive(Serialize)]
    struct User {
        username: String,
        email: String,
        password: String,
    }

    Ok(())
}
