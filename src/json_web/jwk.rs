use std::collections::HashMap;

use base64::{engine::general_purpose, Engine};
use jsonwebtoken::{decode, Algorithm, DecodingKey, TokenData, Validation};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct KeycloakIDClaims {
    pub exp: usize,        // Expiration time in unix epoch
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
    at_hash: String,       // Access Token's hash
    email_verified: bool,
    name: String,
    pub preferred_username: String,
    given_name: String,
    family_name: String,
    pub email: String,
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
pub enum VerificationError {
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

type VerificationResult<T> = Result<T, VerificationError>;

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

fn parse_header(jwt: &str) -> VerificationResult<jsonwebtoken::Header> {
    // JWTs without '.'s are not valid
    let Some((header_b64, _)) = jwt.split_once(".") else {
        return Err(VerificationError::InvalidJwtFormat);
    };

    Ok(serde_json::from_slice::<jsonwebtoken::Header>(
        &general_purpose::STANDARD_NO_PAD.decode(header_b64).unwrap(),
    )?)
}

pub fn verfy_jwt_hmac<T>(jwt: &str, audience: &str, hmac: &str) -> VerificationResult<TokenData<T>>
where
    T: DeserializeOwned,
{
    let header = parse_header(jwt)?;
    if let Algorithm::HS256 = header.alg {
        let decoding_key = DecodingKey::from_secret(hmac.as_bytes());
        let mut validation = Validation::new(header.alg);
        validation.set_audience(&[audience]);
        // Decode the token and get the claims
        Ok(decode::<T>(&jwt, &decoding_key, &validation)?)
    } else {
        Err(VerificationError::UnknownAlgorithm)
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
    let header = parse_header(jwt)?;

    let Some(jwk) = jwks.get(algorithm_to_str(&header.alg)) else {
        return Err(VerificationError::UnknownAlgorithm);
    };

    let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)?;
    let mut validation = Validation::new(header.alg);
    validation.set_audience(&[audience]);

    // Decode the token and get the claims
    Ok(decode::<T>(&jwt, &decoding_key, &validation)?)
}
