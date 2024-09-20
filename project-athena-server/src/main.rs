#![allow(unused)]

use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(root))
        .route("/users", post(create_user).get(get_user))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();

    async fn root() -> &'static str {
        "Hello, World!"
    }

    async fn create_user(Json(payload): Json<CreateUser>) -> (StatusCode, Json<User>) {
        // insert your application logic here
        let user = User {
            id: 27,
            username: payload.username,
        };

        (StatusCode::CREATED, Json(user))
    }

    async fn get_user() -> (StatusCode, Json<User>) {
        let user = User {
            id: 1337,
            username: String::from("Kobe Bryant"),
        };

        (StatusCode::OK, Json(user))
    }

    #[derive(Deserialize)]
    struct CreateUser {
        username: String,
    }

    #[derive(Serialize)]
    struct User {
        id: u64,
        username: String,
    }
}
