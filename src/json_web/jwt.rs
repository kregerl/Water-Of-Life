use std::time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH};

use jsonwebtoken::{EncodingKey, Header};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::WaterOfLifeState;

use super::jwk::verfy_jwt_hmac;

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    #[serde(flatten)]
    common: TokenClaims,
    version: i64,
}

impl RefreshTokenClaims {
    pub fn new(aud: &str, sub: &str, expiration: JWTExpiration<usize>) -> Self {
        Self {
            common: TokenClaims::new(aud, sub, expiration),
            version: 1,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    aud: String,     // Optional. Audience
    exp: usize, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: usize, // Optional. Issued at (as UTC timestamp)
    iss: String, // Optional. Issuer
    pub sub: String, // Optional. Subject (whom token refers to)
}

impl TokenClaims {
    pub fn new(aud: &str, sub: &str, expiration: JWTExpiration<usize>) -> Self {
        Self {
            aud: aud.into(),
            exp: expiration.expires_at,
            iat: expiration.issued_at,
            iss: "http://localhost:3000".into(),
            sub: sub.into(),
        }
    }
}

pub enum TokenState {
    Valid(String),
    Invalid,
    RequiresRefresh(String, User),
}

#[derive(Debug, FromRow)]
pub struct User {
    pub user_id: String,
    pub preferred_username: String,
    pub email: String,
    pub refresh_token_version: i64,
}

pub async fn verify_tokens(
    access_token: &str,
    refresh_token: &str,
    state: &WaterOfLifeState,
) -> TokenState {
    if let Ok(access_token_claims) = verfy_jwt_hmac::<TokenClaims>(
        access_token,
        &state.client_id,
        &state.access_token_hmac_secret,
    ) {
        return TokenState::Valid(access_token_claims.claims.sub);
    }

    if let Ok(refresh_token_claims) = verfy_jwt_hmac::<RefreshTokenClaims>(
        refresh_token,
        &state.client_id,
        &state.refresh_token_hmac_secret,
    ) {
        let maybe_user = sqlx::query_file_as!(
            User,
            "sql/select_user_refresh_token_version.sql",
            refresh_token_claims.claims.common.sub
        )
        .fetch_one(&state.database)
        .await;

        if maybe_user.is_err() {
            return TokenState::Invalid;
        }

        let user = maybe_user.unwrap();
        if refresh_token_claims.claims.version == user.refresh_token_version {
            return TokenState::RequiresRefresh(refresh_token_claims.claims.common.sub, user);
        }
    }

    TokenState::Invalid
}

fn generate_token(
    secret: &str,
    client_id: &str,
    subject: &str,
    expires_in: Duration,
) -> Option<String> {
    let token_encoding_key = EncodingKey::from_secret(secret.as_bytes());
    let token_expiration = calculate_expiration(expires_in).ok()?;
    let token_claims = TokenClaims::new(client_id, subject, token_expiration.clone());
    jsonwebtoken::encode(&Header::default(), &token_claims, &token_encoding_key).ok()
}

pub fn generate_access_and_refresh_tokens(
    access_token_secret: &str,
    refresh_token_secret: &str,
    client_id: &str,
    subject: &str,
) -> Option<(String, String)> {
    let access_token = generate_token(
        access_token_secret,
        client_id,
        subject,
        Duration::from_secs(60 * 30),
    )?;
    tracing::debug!("Generated access token: {}", access_token);

    let refresh_token = generate_token(
        refresh_token_secret,
        client_id,
        subject,
        Duration::from_secs(60 * 60 * 24 * 30),
    )?;
    tracing::debug!("Generated refresh token: {}", refresh_token);

    Some((access_token, refresh_token))
}


#[derive(Debug, Clone)]
struct JWTExpiration<T> {
    pub issued_at: T,
    pub expires_at: T,
}

/// Returns a tuple of (iat, exp).
fn calculate_expiration(expires_in: Duration) -> Result<JWTExpiration<usize>, SystemTimeError> {
    let time_since_epoch = SystemTime::now().duration_since(UNIX_EPOCH)?;
    let exp = time_since_epoch
        .checked_add(expires_in)
        .unwrap_or_else(|| time_since_epoch)
        .as_secs() as usize;

    Ok(JWTExpiration {
        issued_at: time_since_epoch.as_secs() as usize,
        expires_at: exp,
    })
}