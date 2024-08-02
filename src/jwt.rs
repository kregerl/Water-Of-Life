use std::collections::HashMap;

use base64::{engine::general_purpose, Engine};
use jsonwebtoken::{decode, Algorithm, DecodingKey, TokenData, Validation};
use openssl::x509::X509;
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
    sub: String,           // Subject (unique ID per user)
    typ: String,           // Type of token
    azp: String,           // Authorized party (CLIENT_ID)
    pub nonce: String,     // Nonce generated in initial request
    session_state: String, // Session State
    acr: String,           // Authentication context class
    sid: String,           // Session ID
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
    n: Option<String>, // The modulus value if using RSA
    e: Option<String>, // The exponent value if using RSA
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
    #[error("Error validating constructing X509 certificate")]
    OpensslError(#[from] openssl::error::ErrorStack),
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
        &general_purpose::STANDARD.decode(header_b64)?,
    )?;

    let jwk = if let Some(jwk) = jwks.get(algorithm_to_str(&header.alg)) {
        jwk
    } else {
        return Err(JwtVerificationError::UnknownAlgorithm);
    };

    // The first certificate in the X509 chain is always used
    let x5c_certificate = &jwk.x5c[0];

    // Decode the certificate and get the X509 public key
    let der_cert = general_purpose::STANDARD.decode(x5c_certificate)?;
    let cert = X509::from_der(&der_cert)?;
    let public_key = cert.public_key()?;
    let pem = public_key.public_key_to_pem()?;

    let decoding_key = DecodingKey::from_rsa_pem(&pem)?;
    let mut validation = Validation::new(header.alg);
    validation.set_audience(&[audience]);

    // Decode the token and get the claims
    Ok(decode::<T>(&jwt, &decoding_key, &validation)?)
}
