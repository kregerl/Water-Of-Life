use serde::{Deserialize, Serialize};

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
    nonce: String,         // Nonce generated in initial request
    session_state: String, // Session State
    acr: String,           // Authentication context class
    sid: String,           // Session ID
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeycloakIDClaims {
    #[serde(flatten)]
    common: CommonClaims,
    at_hash: String,       // Access Token's hash
    email_verified: bool,
    name: String,
    preferred_username: String,
    given_name: String,
    family_name: String,
    email: String,
}
