use std::collections::HashMap;
use std::env;

use axum::extract::{MatchedPath, Request};
use axum::handler::HandlerWithoutStateExt;
use axum::{routing::get, Router};
use jwt::JWKCertificate;
use reqwest::{Client, StatusCode};
use routes::oidc::{get_jwks, get_well_known_configuration, OpenidConfiguration};
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tower_sessions::cookie::time::Duration;
use tower_sessions::cookie::SameSite;
use tower_sessions::{MemoryStore, SessionManagerLayer};

mod jwt;
mod routes;

#[derive(Clone)]
struct WaterOfLifeState {
    client: reqwest::Client,
    database: SqlitePool,
    oidc_configuration: OpenidConfiguration,
    client_id: String,
    client_secret: String,
    jwks: HashMap<String, JWKCertificate>,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let database = SqlitePool::connect_with(
        SqliteConnectOptions::new()
            .filename("test.db")
            .create_if_missing(true),
    )
    .await
    .unwrap();

    let client_id = env::var("CLIENT_ID").unwrap();
    let client_secret = env::var("CLIENT_SECRET").unwrap();

    let client = Client::new();

    let oidc_configuration = get_well_known_configuration(&client).await.unwrap();
    let jwks = get_jwks(&oidc_configuration.jwks_uri, &client)
        .await
        .unwrap();

    let state = WaterOfLifeState {
        client,
        database,
        oidc_configuration,
        client_id,
        client_secret,
        jwks,
    };

    // Probably fine to store nonces in memory for now since theyre 32 bytes each
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_same_site(SameSite::Lax)
        // FIXME: This should be removed once the web server is running HTTPS
        .with_secure(false)
        .with_expiry(tower_sessions::Expiry::OnInactivity(Duration::minutes(2)));

    let app = Router::new()
        .route("/oidc/login", get(routes::login))
        .route("/oidc/logout", get(routes::logout))
        .route("/oidc/token", get(routes::token))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(create_span)
                .on_failure(()),
        )
        .layer(session_layer)
        .fallback_service(
            ServeDir::new("./frontend/build").not_found_service(handle_error.into_service()),
        )
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

fn create_span(request: &Request) -> tracing::Span {
    let method = request.method();
    let uri = request.uri();

    let matched_path = request
        .extensions()
        .get::<MatchedPath>()
        .map(|matched_path| matched_path.as_str())
        .unwrap_or("<unknown>");

    tracing::debug_span!("request", %method, %uri, matched_path)
}

#[allow(clippy::unused_async)]
async fn handle_error() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "That endpoint does not exist.")
}
