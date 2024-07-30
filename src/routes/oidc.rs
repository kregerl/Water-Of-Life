use core::str;

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
    Json,
};
use base64::{engine::general_purpose, Engine};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use openssl::x509::X509;
use reqwest::{header::CONTENT_TYPE, StatusCode};
use serde::{Deserialize, Serialize};
use textnonce::TextNonce;
use thiserror::Error;
use tower_sessions::Session;
use url::Url;

use crate::{routes::jwt::KeycloakIDClaims, WaterOfLifeState};

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

#[derive(Clone, Debug, Deserialize)]
pub struct OpenidConfiguration {
    issuer: String,
    authorization_endpoint: String,
    token_endpoint: String,
    userinfo_endpoint: String,
    end_session_endpoint: String,
    jwks_uri: String,
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

#[derive(Debug, Deserialize)]
pub struct AuthCode {
    session_state: String,
    iss: String,
    code: String,
}

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
    tracing::info!(
        "Session nonce: {:#?}",
        nonce
    );

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

    let x5c_certificate = "MIIClzCCAX8CBgGOoSUh5zANBgkqhkiG9w0BAQsFADAPMQ0wCwYDVQQDDARtYWluMB4XDTI0MDQwMjIzMjcyOVoXDTM0MDQwMjIzMjkwOVowDzENMAsGA1UEAwwEbWFpbjCCASIwDQYJKoZIhvcNAQEBBQADggEPADCCAQoCggEBAKjaxHqgbDNpnfw3Z+AVnseA+zV9kPwxnmGpQJtxQOTx3kIDRpfIAIlBR1IkkiIktb2PtRWt8b4QvyaU1gjLfEO/9jA9Do6+hAxhB9SC5maSO/TckWXvT7GHBbqlAtemBvR11IiudIfCXNoszUaGdCYwyK6l7fpu2l90sih+9dapEBXKVv/ayoyub7o9mHmXqIsPpqMISR/J2sAIm5RSHsFQhvOkf3QSduaBvBCyN3NVVaSfwSYXDthjKZL3ayYFu+Yx4xumRe2+/HkhHjUTrtCLAHpmrrz2A5ouBIaGtq2wRvoRVFKgl+EAnDdpPx36EteVfSwkFcklgFzjresYD5cCAwEAATANBgkqhkiG9w0BAQsFAAOCAQEAG2Wf4pYDBv+yydmPuNf9SKDEBg8UR3a1lkGKuVfSCAvdLg+2aoDldeyG5IuT791KP6J+DgauY5V/uwXFBXReAAeiy6c/DgM5zk8qEUUvSHPlTu4yCs/3bxHEtJuc23/bWuo3eQlcAkJxCxkU+i1oSqkPI8EQz3no6zVtvLAw4OcoKh2XkMPTWpJ3WTPpFQWBIZ7ulj1QUxilMlKUgLQZSydOGHAqbx0NKKvADX1t4jdj/nIIuvPpQf1dL0MVUetSkhc9H70I+FMjXqC+Yp9lpj6eYjqSIyO1cm+XNxNw+fczvtiCMbs5O8g3bt8jM/wCHakIZPuc722B9QfKslFgqg==";
    let der_cert = general_purpose::STANDARD.decode(x5c_certificate).unwrap();
    let cert = X509::from_der(&der_cert).unwrap();
    let public_key = cert.public_key().unwrap();
    let pem = public_key.public_key_to_pem().unwrap();
    let decoding_key = DecodingKey::from_rsa_pem(&pem).unwrap();

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[state.client_id.as_str()]);

    // Decode the token and get the claims
    let token_data = decode::<KeycloakIDClaims>(&tokens.id_token, &decoding_key, &validation);

    let path = match token_data {
        Ok(data) => {
            tracing::debug!("Token is valid: {:?}", data.claims);
            "/"
        },
        Err(err) => {
            tracing::debug!("Invalid token: {:?}", err);
            "/login"
        },
    };

    Ok(Redirect::to(path).into_response())
}

#[test]
fn test() {
    let token = "<token>";
    let x5c_certificate = "MIIClzCCAX8CBgGOoSUh5zANBgkqhkiG9w0BAQsFADAPMQ0wCwYDVQQDDARtYWluMB4XDTI0MDQwMjIzMjcyOVoXDTM0MDQwMjIzMjkwOVowDzENMAsGA1UEAwwEbWFpbjCCASIwDQYJKoZIhvcNAQEBBQADggEPADCCAQoCggEBAKjaxHqgbDNpnfw3Z+AVnseA+zV9kPwxnmGpQJtxQOTx3kIDRpfIAIlBR1IkkiIktb2PtRWt8b4QvyaU1gjLfEO/9jA9Do6+hAxhB9SC5maSO/TckWXvT7GHBbqlAtemBvR11IiudIfCXNoszUaGdCYwyK6l7fpu2l90sih+9dapEBXKVv/ayoyub7o9mHmXqIsPpqMISR/J2sAIm5RSHsFQhvOkf3QSduaBvBCyN3NVVaSfwSYXDthjKZL3ayYFu+Yx4xumRe2+/HkhHjUTrtCLAHpmrrz2A5ouBIaGtq2wRvoRVFKgl+EAnDdpPx36EteVfSwkFcklgFzjresYD5cCAwEAATANBgkqhkiG9w0BAQsFAAOCAQEAG2Wf4pYDBv+yydmPuNf9SKDEBg8UR3a1lkGKuVfSCAvdLg+2aoDldeyG5IuT791KP6J+DgauY5V/uwXFBXReAAeiy6c/DgM5zk8qEUUvSHPlTu4yCs/3bxHEtJuc23/bWuo3eQlcAkJxCxkU+i1oSqkPI8EQz3no6zVtvLAw4OcoKh2XkMPTWpJ3WTPpFQWBIZ7ulj1QUxilMlKUgLQZSydOGHAqbx0NKKvADX1t4jdj/nIIuvPpQf1dL0MVUetSkhc9H70I+FMjXqC+Yp9lpj6eYjqSIyO1cm+XNxNw+fczvtiCMbs5O8g3bt8jM/wCHakIZPuc722B9QfKslFgqg==";
    // Decode the base64-encoded certificate to DER format
    let der_cert = base64::engine::general_purpose::STANDARD
        .decode(x5c_certificate)
        .expect("Invalid base64 in x5c");

    // Parse the certificate using openssl
    let cert = X509::from_der(&der_cert).expect("Failed to parse X509 certificate");

    // Extract the public key from the certificate
    let public_key = cert.public_key().expect("Failed to extract public key");

    // Convert the public key to PEM format
    let pem = public_key
        .public_key_to_pem()
        .expect("Failed to convert to PEM");

    // Verify the JWT using the extracted public key
    let decoding_key = DecodingKey::from_rsa_pem(&pem).expect("Invalid PEM key");

    // Validate the token (use proper validation as required)
    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = false;
    validation.set_audience(&["wateroflife"]);


    // Decode the token and get the claims
    let token_data = decode::<KeycloakIDClaims>(&token, &decoding_key, &validation);

    match token_data {
        Ok(data) => println!("Token is valid: {:?}", data.claims),
        Err(err) => println!("Invalid token: {:?}", err),
    }
}
