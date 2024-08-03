use std::{
    collections::HashMap,
    time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH},
};

use base64::{engine::general_purpose, Engine};
use jsonwebtoken::{decode, Algorithm, DecodingKey, TokenData, Validation};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct CommonClaims {
    exp: usize,            // Expiration time un unix epoch
    iat: usize,            // Issued at time in unix epoch
    auth_time: usize,      // Time when authentication occured un unix epoch
    jti: String,           // JWT Unique ID
    iss: String,           // Issuer
    aud: String,           // Audience
    pub sub: String,       // Subject (unique ID per user)
    typ: String,           // Type of token
    azp: String,           // Authorized party (CLIENT_ID)
    pub nonce: String,     // Nonce generated in initial request
    session_state: String, // Session State
    acr: String,           // Authentication context class
    sid: String,           // Session ID
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppIDClaims {
    aud: String, // Optional. Audience
    exp: usize, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: usize, // Optional. Issued at (as UTC timestamp)
    iss: String, // Optional. Issuer
    sub: String, // Optional. Subject (whom token refers to)
}

impl AppIDClaims {
    pub fn new(aud: &str, sub: &str) -> Result<Self, SystemTimeError> {
        let time_since_epoch = SystemTime::now().duration_since(UNIX_EPOCH)?;
        let exp = time_since_epoch
            .checked_add(Duration::from_secs(60 * 30))
            .unwrap_or_else(|| time_since_epoch)
            .as_secs() as usize;
        Ok(Self {
            aud: aud.into(),
            exp,
            iat: time_since_epoch.as_secs() as usize,
            iss: "http://localhost:3000".into(),
            sub: sub.into(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeycloakIDClaims {
    #[serde(flatten)]
    pub common: CommonClaims,
    at_hash: String, // Access Token's hash
    email_verified: bool,
    name: String,
    preferred_username: String,
    given_name: String,
    family_name: String,
    email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JWKCertificate {
    kid: String,     // Key ID
    kty: String,     // Key Type
    pub alg: String, // Algorithm used
    #[serde(rename = "use")]
    used_for: String, // What the key is used for ("enc", "sig")
    n: String, // The modulus value if using RSA - Might not always exist if using another provider
    e: String, // The exponent value if using RSA - Might not always exist if using another provider
    pub x5c: Vec<String>, // The X509 certificate chain - The first entry in the array should always be used for token verification
    x5t: String,          // The X509 thumbprint
    #[serde(rename = "x5t#S256")]
    x5t_hash: String, // The SHA256 hash of the thumbprint
}

#[derive(Error, Debug)]
pub enum JwtVerificationError {
    #[error("Invalid json web token format")]
    InvalidJwtFormat,
    #[error("Unknown JWK algorithm")]
    UnknownAlgorithm,
    #[error("Error decoding data from base64")]
    Base64DecodeError(#[from] base64::DecodeError),
    #[error("Error parsing json")]
    JsonParseError(#[from] serde_json::Error),
    #[error("Error validating JWT signature")]
    InvalidSignature(#[from] jsonwebtoken::errors::Error),
}

type VerificationResult<T> = Result<T, JwtVerificationError>;

fn algorithm_to_str(algorithm: &Algorithm) -> &str {
    match algorithm {
        Algorithm::HS256 => "HS256",
        Algorithm::HS384 => "HS384",
        Algorithm::HS512 => "HS512",
        Algorithm::ES256 => "ES256",
        Algorithm::ES384 => "ES384",
        Algorithm::RS256 => "RS256",
        Algorithm::RS384 => "RS384",
        Algorithm::PS256 => "PS256",
        Algorithm::PS384 => "PS384",
        Algorithm::PS512 => "PS512",
        Algorithm::RS512 => "RS512",
        Algorithm::EdDSA => "EdDSA",
    }
}

pub fn verify_jwt<T>(
    jwt: &str,
    audience: &str,
    jwks: &HashMap<String, JWKCertificate>,
) -> VerificationResult<TokenData<T>>
where
    T: DeserializeOwned,
{
    // JWTs without '.'s are not valid
    let header_b64 = if let Some((header_b64, _)) = jwt.split_once(".") {
        header_b64
    } else {
        return Err(JwtVerificationError::InvalidJwtFormat);
    };

    let header = serde_json::from_slice::<jsonwebtoken::Header>(
        &general_purpose::STANDARD_NO_PAD.decode(header_b64).unwrap(),
    )?;

    let jwk = if let Some(jwk) = jwks.get(algorithm_to_str(&header.alg)) {
        jwk
    } else {
        return Err(JwtVerificationError::UnknownAlgorithm);
    };

    let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)?;
    let mut validation = Validation::new(header.alg);
    validation.set_audience(&[audience]);

    // Decode the token and get the claims
    Ok(decode::<T>(&jwt, &decoding_key, &validation)?)
}
