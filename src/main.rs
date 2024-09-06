use std::collections::HashMap;
use std::path::PathBuf;
use std::{env, fs};

use axum::handler::HandlerWithoutStateExt;
use axum::routing::{post, put, MethodRouter};
use axum::{routing::get, Router};
use json_web::JWKCertificate;
use reqwest::Client;
use services::{get_jwks, get_well_known_configuration, OpenidConfiguration};
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

mod cookie;
mod json_web;
mod middleware;
mod services;

#[derive(Clone)]
struct WaterOfLifeState {
    client: reqwest::Client,
    database: SqlitePool,
    oidc_configuration: OpenidConfiguration,
    images_path: PathBuf,
    client_id: String,
    client_secret: String,
    access_token_hmac_secret: String,
    refresh_token_hmac_secret: String,
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

    sqlx::migrate!("./migrations").run(&database).await.unwrap();

    let client_id =
        env::var("CLIENT_ID").expect("Expected the 'CLIENT_ID' environment variable to be set.");
    let client_secret = env::var("CLIENT_SECRET")
        .expect("Expected the 'CLIENT_SECRET' environment variable to be set.");
    let access_token_hmac_secret = env::var("ACCESS_TOKEN_HMAC_SECRET")
        .expect("Expected the 'ACCESS_TOKEN_HMAC_SECRET' environment variable to be set.");
    let refresh_token_hmac_secret = env::var("REFRESH_TOKEN_HMAC_SECRET")
        .expect("Expected the 'REFRESH_TOKEN_HMAC_SECRET' environment variable to be set.");

    let client = Client::new();

    let oidc_configuration = get_well_known_configuration(&client).await.unwrap();
    let jwks = get_jwks(&oidc_configuration.jwks_uri, &client)
        .await
        .unwrap();

    let images_path = PathBuf::new().join("./spirit_images");
    fs::create_dir_all(&images_path).unwrap();

    let state = WaterOfLifeState {
        client,
        database,
        oidc_configuration,
        images_path,
        client_id,
        client_secret,
        access_token_hmac_secret,
        refresh_token_hmac_secret,
        jwks,
    };

    let app = Router::new()
        .route("/api/spirit", post(services::add_spirit))
        .route("/api/spirit/search", get(services::search_spirit))
        .route("/api/spirit/:id", put(services::edit_spirit))
        .route("/api/spirit/:id/image", put(services::upload_spirit_image))
        .route("/api/spirit/:id/image", get(services::get_spirit_image))
        .route("/api/user_info", get(services::user_info))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::authentication,
        ))
        .route("/oidc/login", get(services::login))
        .route("/oidc/logout", get(services::logout))
        .route("/oidc/token", get(services::token))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(middleware::create_span)
                .on_failure(()),
        )
        .layer(middleware::session_layer())
        .layer(CookieManagerLayer::new())
        .fallback_service(
            ServeDir::new("./frontend/build")
                .not_found_service(middleware::handle_error.into_service()),
        )
        // TODO: Make some authentication middleware
        // https://docs.rs/axum/latest/axum/middleware/index.html#passing-state-from-middleware-to-handlers
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
