use core::str;
use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::{IntoResponse, Redirect, Response},
    Json,
};
use jsonwebtoken::TokenData;
use reqwest::{header::CONTENT_TYPE, Client, StatusCode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sqlx::SqlitePool;
use textnonce::TextNonce;
use thiserror::Error;
use tower_cookies::{cookie::SameSite, Cookie, Cookies};
use tower_sessions::Session;
use url::Url;

use crate::{
    json_web::{
        generate_access_and_refresh_tokens, verify_jwt, verify_tokens, JWKCertificate,
        KeycloakIDClaims, TokenState,
    },
    WaterOfLifeState,
};

pub const REALM_URL: &'static str = "https://sso.loucaskreger.com/realms/main";
pub const WELL_KNOWN_CONFIGURATION_ENDPOINT: &'static str = ".well-known/openid-configuration";

const NONCE_SESSION_KEY: &'static str = "nonce";
const REDIRECT_URI: &'static str = "http://localhost:3000/oidc/token";

#[derive(Error, Debug)]
pub enum AuthenticationError {
    #[error("Unknown authentication error, try again later")]
    Internal,
    #[error("Could not authenticate with json web token")]
    Error(String),
    #[error("Error issuing authentication requests")]
    HttpError(#[from] reqwest::Error),
    #[error("Error formatting URI")]
    ParseError(#[from] url::ParseError),
    #[error("Error storing session data")]
    SessionStorage(#[from] tower_sessions::session::Error),
    #[error("Error deserializing json.")]
    Deserialization(#[from] serde_json::Error),
}

impl IntoResponse for AuthenticationError {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }

        let mut status = StatusCode::INTERNAL_SERVER_ERROR;

        match self {
            Self::HttpError(e) => tracing::error!("{}", e),
            Self::ParseError(e) => tracing::error!("{}", e),
            Self::SessionStorage(e) => tracing::error!("{}", e),
            Self::Deserialization(e) => tracing::error!("{}", e),
            Self::Internal => {}
            Self::Error(e) => {
                tracing::warn!("{}", e);
                status = StatusCode::UNAUTHORIZED;
            }
        }
        (
            status,
            Json(ErrorResponse {
                message: "Please try again later".into(),
            }),
        )
            .into_response()
    }
}

pub type AuthenticationResult<T> = Result<T, AuthenticationError>;

#[allow(unused)]
#[derive(Clone, Debug, Deserialize)]
pub struct OpenidConfiguration {
    issuer: String,
    authorization_endpoint: String,
    token_endpoint: String,
    userinfo_endpoint: String,
    end_session_endpoint: String,
    pub jwks_uri: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Nonce(String);

pub async fn login(
    session: Session,
    State(state): State<WaterOfLifeState>,
) -> AuthenticationResult<Redirect> {
    let nonce = match TextNonce::sized(32) {
        Ok(nonce) => nonce,
        Err(e) => {
            tracing::error!("{}", e);
            return Err(AuthenticationError::Internal);
        }
    }
    .0;
    tracing::info!("Nonce: {}", nonce);
    session
        .insert(NONCE_SESSION_KEY, Nonce(nonce.clone()))
        .await?;
    let url = Url::parse_with_params(
        &state.oidc_configuration.authorization_endpoint,
        &[
            ("client_id", state.client_id.as_str()),
            ("redirect_uri", REDIRECT_URI),
            ("response_type", "code"),
            ("scope", "openid roles"),
            ("nonce", &nonce),
        ],
    )?;
    tracing::debug!("Generated URL: {}", url.as_str());

    let redirect = Redirect::to(url.as_str());
    Ok(redirect)
}

pub async fn logout(
    headers: HeaderMap,
    cookies: Cookies,
    State(state): State<WaterOfLifeState>,
) -> AuthenticationResult<()> {
    tracing::info!("Cookies: {:#?}", cookies.get("wl_id").unwrap().value());
    let cookie = cookies.get("wl_id").unwrap().value();
    let client = Client::new();
    // client
    //     .post("https://sso.loucaskreger.com/realms/main/protocol/openid-connect/logout")
    //     .query(&[("client_id", "CLIENT_ID")])
    //     .send()
    //     .await;
    Ok(())
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct AuthCode {
    session_state: String,
    iss: String,
    code: String,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u32,
    refresh_expires_in: u32,
    refresh_token: String,
    token_type: String,
    id_token: String,
    session_state: String,
    scope: String,
}

pub async fn token(
    session: Session,
    cookies: Cookies,
    State(state): State<WaterOfLifeState>,
    Query(query_params): Query<AuthCode>,
) -> AuthenticationResult<Response> {
    let nonce = session.get::<Nonce>(NONCE_SESSION_KEY).await?;
    tracing::info!("Session nonce: {:#?}", nonce);

    tracing::debug!("auth_response: {:#?}", query_params);
    let response = state
        .client
        .post(state.oidc_configuration.token_endpoint)
        .form(&[
            ("client_id", state.client_id.as_str()),
            ("client_secret", state.client_secret.as_str()),
            ("code", &query_params.code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", REDIRECT_URI),
        ])
        .header(CONTENT_TYPE, "x-www-form-urlencoded")
        .send()
        .await?;

    let tokens: TokenResponse = response.json().await?;
    tracing::debug!("Got tokens: {:#?}", tokens);

    let endpoint: &'static str =
        match verify_jwt::<KeycloakIDClaims>(&tokens.id_token, &state.client_id, &state.jwks) {
            Ok(token_data) => {
                let maybe_tokens = generate_access_and_refresh_tokens(
                    &state.access_token_hmac_secret,
                    &state.refresh_token_hmac_secret,
                    &state.client_id,
                    &token_data.claims.sub,
                );

                if let Some((access_token, refresh_token)) = maybe_tokens {
                    let _ = insert_user(&state.database, &token_data).await.unwrap();

                    // FIXME: Replace with axum's CookieJar which must be returned from the handler.
                    cookies.add(create_token_cookie("wl_id", access_token));
                    cookies.add(create_token_cookie("wl_rid", refresh_token));

                    "/"
                } else {
                    "/login"
                }
            }
            Err(error) => {
                tracing::info!("{}", error);
                "/login"
            }
        };

    Ok(Redirect::to(endpoint).into_response())
}

fn create_token_cookie<'a>(key: &'a str, token: String) -> Cookie<'a> {
    let mut cookie = Cookie::new(key, token);
    cookie.set_path("/");
    cookie.set_secure(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_http_only(true);
    cookie
}
async fn insert_user(
    database: &SqlitePool,
    data: &TokenData<KeycloakIDClaims>,
) -> AuthenticationResult<()> {
    let x = sqlx::query_file!(
        "sql/insert_user.sql",
        data.claims.sub,
        data.claims.preferred_username,
        data.claims.email,
        1
    )
    .execute(database)
    .await
    .unwrap();

    Ok(())
}

async fn get_as_json<T>(client: &Client, url: &str) -> AuthenticationResult<T>
where
    T: DeserializeOwned,
{
    Ok(client.get(url).send().await?.json::<T>().await?)
}

pub async fn get_well_known_configuration(
    client: &Client,
) -> AuthenticationResult<OpenidConfiguration> {
    get_as_json(
        client,
        &format!("{}/{}", REALM_URL, WELL_KNOWN_CONFIGURATION_ENDPOINT),
    )
    .await
}

pub async fn get_jwks(
    jwks_uri: &str,
    client: &Client,
) -> AuthenticationResult<HashMap<String, JWKCertificate>> {
    let mut response = get_as_json::<HashMap<String, serde_json::Value>>(client, jwks_uri).await?;

    let keys = response.remove("keys").unwrap();
    Ok(serde_json::from_value::<Vec<JWKCertificate>>(keys)?
        .into_iter()
        .map(|cert| (cert.alg.clone(), cert))
        .collect::<HashMap<String, JWKCertificate>>())
}
