use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tower_cookies::{Cookie, Cookies};

use crate::{web::AUTH_TOKEN, Error, Result};

pub fn routes() -> Router {
    Router::new().route("/api/login", post(api_login))
}

async fn api_login(cookies: Cookies, payload: Json<LoginPayload>) -> Result<Json<Value>> {
    tracing::info!("->> {:12} - api login", "Handler");

    if (payload.username != "demo1" || payload.password != "welcome") {
        tracing::error!("login error");
        return Err(Error::LoginFail);
    }

    cookies.add(Cookie::new(AUTH_TOKEN, "bla"));

    let body = Json(json!({
        "result": {
            "success": true,
        }
    }));

    let username = payload.username.as_str();

    tracing::info!("Login in successfully! username: {username}");

    Ok(body)
}

#[derive(Deserialize, Serialize, Debug)]
struct LoginPayload {
    username: String,
    password: String,
}
