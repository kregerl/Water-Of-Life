use core::str;
use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
    Json,
};
use reqwest::{header::CONTENT_TYPE, Client, StatusCode};
use serde::{Deserialize, Serialize};
use textnonce::TextNonce;
use thiserror::Error;
use tower_sessions::Session;
use url::Url;

use crate::{
    jwt::{verify_jwt, AppIDClaims, JWKCertificate, KeycloakIDClaims},
    WaterOfLifeState,
};

pub const REALM_URL: &'static str = "https://sso.loucaskreger.com/realms/main";
pub const WELL_KNOWN_CONFIGURATION_ENDPOINT: &'static str = ".well-known/openid-configuration";

const NONCE_SESSION_KEY: &'static str = "nonce";
const REDIRECT_URI: &'static str = "http://localhost:3000/oidc/token";

#[derive(Error, Debug)]
pub enum AuthenticationError {
    #[error("Unknown authentication error, try again later")]
    Unknown,
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

        match self {
            Self::HttpError(e) => tracing::error!("{}", e),
            Self::ParseError(e) => tracing::error!("{}", e),
            Self::SessionStorage(e) => tracing::error!("{}", e),
            Self::Deserialization(e) => tracing::error!("{}", e),
            Self::Unknown => {}
        }
        (
            StatusCode::INTERNAL_SERVER_ERROR,
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
            return Err(AuthenticationError::Unknown);
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

pub async fn logout(State(state): State<WaterOfLifeState>) -> AuthenticationResult<()> {
    // let client = Client::new();
    // client
    //     .post("https://sso.loucaskreger.com/realms/main/protocol/openid-connect/logout")
    //     .query(&[("client_id", CLIENT_ID)])
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

    let token_data =
        verify_jwt::<KeycloakIDClaims>(&tokens.id_token, &state.client_id, &state.jwks);

    let path = match token_data {
        Ok(data) if nonce.is_some() => {
            let nonce = nonce.unwrap();
            // If nonce doesn't match - the request is invalid
            if data.claims.common.nonce == nonce.0 {
                tracing::debug!("Token is valid: {:?}", data.claims);
                let app_claims = AppIDClaims::new(&state.client_id, "loucas").unwrap();
                let app_token = jsonwebtoken::encode(
                    &jsonwebtoken::Header::default(),
                    &app_claims,
                    &jsonwebtoken::EncodingKey::from_secret("secret".as_ref()),
                )
                .unwrap();
                tracing::debug!("Generated new token: {}", app_token);
                // let user_id = &data.claims.common.sub;
                "/"
            } else {
                tracing::debug!("Nonce does not match the expected value");
                "/login"
            }
        }
        Ok(_) => {
            tracing::debug!("Could not verify nonce it does not exist in session storage.");
            "/login"
        }
        Err(err) => {
            tracing::debug!("Invalid token: {:?}", err);
            "/login"
        }
    };

    Ok(Redirect::to(path).into_response())
}

pub async fn get_well_known_configuration(
    client: &Client,
) -> AuthenticationResult<OpenidConfiguration> {
    Ok(client
        .get(format!(
            "{}/{}",
            REALM_URL, WELL_KNOWN_CONFIGURATION_ENDPOINT
        ))
        .send()
        .await?
        .json()
        .await?)
}

pub async fn get_jwks(
    jwks_uri: &str,
    client: &Client,
) -> AuthenticationResult<HashMap<String, JWKCertificate>> {
    let mut response = client
        .get(jwks_uri)
        .send()
        .await?
        .json::<HashMap<String, serde_json::Value>>()
        .await?;
    let keys = response.remove("keys").unwrap();
    Ok(serde_json::from_value::<Vec<JWKCertificate>>(keys)?
        .into_iter()
        .map(|cert| (cert.alg.clone(), cert))
        .collect::<HashMap<String, JWKCertificate>>())
}
