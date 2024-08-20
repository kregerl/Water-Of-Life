use std::time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH};

use jsonwebtoken::{EncodingKey, Header};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::WaterOfLifeState;

use super::jwk::verfy_jwt_hmac;

trait Claim {
    fn new(aud: &str, sub: &str, role: &str, expiration: JWTExpiration<usize>) -> Self;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommonClaims {
    aud: String,     // Optional. Audience
    exp: usize, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: usize, // Optional. Issued at (as UTC timestamp)
    iss: String, // Optional. Issuer
    pub sub: String, // Optional. Subject (whom token refers to)
}

impl CommonClaims {
    fn new(aud: &str, sub: &str, expiration: JWTExpiration<usize>) -> Self {
        Self {
            aud: aud.into(),
            exp: expiration.expires_at,
            iat: expiration.issued_at,
            iss: "http://localhost:3000".into(),
            sub: sub.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    #[serde(flatten)]
    common: CommonClaims,
    version: i64,
}

impl Claim for RefreshTokenClaims {
    fn new(aud: &str, sub: &str, _role: &str, expiration: JWTExpiration<usize>) -> Self {
        Self {
            common: CommonClaims::new(aud, sub, expiration),
            version: 1,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    #[serde(flatten)]
    common: CommonClaims,
    role: String,
    additional_scopes: Vec<String>,
}

impl Claim for AccessTokenClaims {
    fn new(aud: &str, sub: &str, role: &str, expiration: JWTExpiration<usize>) -> Self {
        Self {
            common: CommonClaims::new(aud, sub, expiration),
            role: role.to_owned(),
            additional_scopes: Vec::new(),
        }
    }
}

pub enum TokenState {
    Valid(String),
    Invalid,
    RequiresRefresh(String, User),
}

#[derive(Debug, FromRow, Clone)]
pub struct User {
    pub user_id: String,
    pub preferred_username: String,
    pub email: String,
    pub refresh_token_version: i64,
    pub role: String,
}

pub async fn verify_tokens(
    access_token: &str,
    refresh_token: &str,
    state: &WaterOfLifeState,
) -> TokenState {
    if let Ok(access_token_claims) = verfy_jwt_hmac::<AccessTokenClaims>(
        access_token,
        &state.client_id,
        &state.access_token_hmac_secret,
    ) {
        return TokenState::Valid(access_token_claims.claims.common.sub);
    }

    if let Ok(refresh_token_claims) = verfy_jwt_hmac::<RefreshTokenClaims>(
        refresh_token,
        &state.client_id,
        &state.refresh_token_hmac_secret,
    ) {
        let maybe_user = sqlx::query_file_as!(
            User,
            "sql/select_user.sql",
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

fn generate_token<T>(
    secret: &str,
    client_id: &str,
    subject: &str,
    role: &str,
    expires_in: Duration,
) -> Option<String>
where
    T: Claim + Serialize,
{
    let token_encoding_key = EncodingKey::from_secret(secret.as_bytes());
    let token_expiration = calculate_expiration(expires_in).ok()?;
    let token_claims = T::new(client_id, subject, role, token_expiration.clone());
    jsonwebtoken::encode(&Header::default(), &token_claims, &token_encoding_key).ok()
}

pub fn generate_access_and_refresh_tokens(
    access_token_secret: &str,
    refresh_token_secret: &str,
    client_id: &str,
    subject: &str,
    role: &str
) -> Option<(String, String)> {
    let access_token = generate_token::<AccessTokenClaims>(
        access_token_secret,
        client_id,
        subject,
        role,
        Duration::from_secs(60 * 30),
    )?;
    tracing::debug!("Generated access token: {}", access_token);

    let refresh_token = generate_token::<RefreshTokenClaims>(
        refresh_token_secret,
        client_id,
        subject,
        role,
        Duration::from_secs(60 * 60 * 24 * 30),
    )?;
    tracing::debug!("Generated refresh token: {}", refresh_token);

    Some((access_token, refresh_token))
}

#[derive(Debug, Clone)]
pub struct JWTExpiration<T> {
    pub issued_at: T,
    pub expires_at: T,
}

/// Returns a tuple of (iat, exp).
pub fn calculate_expiration(expires_in: Duration) -> Result<JWTExpiration<usize>, SystemTimeError> {
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
