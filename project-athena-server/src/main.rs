#![allow(unused)]

use std::net::ToSocketAddrs;

use tower_cookies::CookieManagerLayer;
use tracing::Level;

use argon2::{password_hash::SaltString, Argon2, PasswordHash};
use axum::{
    extract::{self, Path},
    http::StatusCode,
    middleware,
    response::{IntoResponse, Response},
    routing::{get, get_service, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, query};
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{filter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub use self::error::{Error, Result};

mod error;
mod model;
mod web;

#[derive(Deserialize, Debug)]
struct HelloWorld {
    name: Option<String>,
}

#[derive(Deserialize, Debug)]
struct CreateUser {
    username: Option<String>,
    email: Option<String>,
    password: Option<String>,
}

#[derive(Serialize)]
struct User {
    username: String,
    email: String,
    password: String,
}

async fn root() -> impl IntoResponse {
    format!("ROOT!")
}

// example: search params - query string
async fn hello_params(extract::Query(params): extract::Query<HelloWorld>) -> impl IntoResponse {
    tracing::info!("->> {:<12} - handler-hello_params - {params:?}", "HANDLER");

    let name = params.name.as_deref().unwrap_or("World!");

    (StatusCode::OK, format!("Hello {name} from search params"))
}

// example: path
async fn hello_path(Path(name): Path<String>) -> impl IntoResponse {
    tracing::info!("->> {:<12} - handler-hello_path - {name:?}", "HANDLER");

    (StatusCode::OK, format!("Hello {name} from path"))
}

// async fn create_user(extract::Query(params): extract::Query<CreateUser>) -> impl IntoResponse {
//     // insert your application logic here

//     let user = User {
//         username: payload.username,
//         password: payload.password,
//         email: payload.email,
//     };

//     (StatusCode::CREATED, Json(user))
// }

async fn get_user() -> (StatusCode, Json<User>) {
    let user = User {
        username: String::from("Kobe Bryant"),
        email: String::from("Kobe Bryant"),
        password: String::from("Kobe Bryant"),
    };

    (StatusCode::OK, Json(user))
}

fn routes_static() -> Router {
    Router::new().nest_service("/", get_service(ServeDir::new("./")))
}

fn routes_hello() -> Router {
    Router::new()
        .route("/hello", get(hello_params))
        .route("/hello/:name", get(hello_path))
}

async fn main_response_mapper(res: Response) -> Response {
    tracing::info!("->> {:<12} - main_response_mapper", "RES_MAPPER");

    println!();

    res
}

#[tokio::main]
async fn main() {
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

    // let db = PgPoolOptions::new()
    //     // The default connection limit for a Postgres server is 100 connections, minus 3 for superusers.
    //     // Since we're using the default superuser we don't have to worry about this too much,
    //     // although we should leave some connections available for manual access.
    //     //
    //     // If you're deploying your application with multiple replicas, then the total
    //     // across all replicas should not exceed the Postgres connection limit.
    //     .max_connections(50)
    //     .connect("postgresql://admin:secret@localhost/athena_db")
    //     .await?;

    // tracing::info!("DB connect successfully!");

    // sqlx::migrate!("db/migrations/").run(&db).await?;
    let mc = model::ModelController::new().await.unwrap();

    let routes_all = Router::new()
        .merge(routes_hello())
        .merge(web::routes_login::routes())
        .merge(web::routes_ticket::routes(mc));

    let app = Router::new()
        .route("/", get(root))
        .route(
            "/users",
            // post(create_user).
            get(get_user),
        )
        .merge(routes_all)
        .layer(middleware::map_response(main_response_mapper))
        .layer(CorsLayer::permissive())
        .layer(CookieManagerLayer::new())
        .layer(TraceLayer::new_for_http())
        .fallback_service(routes_static());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
