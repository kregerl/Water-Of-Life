use axum::{
    extract::{Query, State}, response::{IntoResponse, Redirect, Response}, Json
};
use serde::Deserialize;
use tower_cookies::Cookies;

use crate::{
    json_web::{verify_tokens, TokenState},
    WaterOfLifeState,
};

use super::oidc::{AuthenticationError, AuthenticationResult};

async fn validate_cookies(
    cookies: Cookies,
    state: &WaterOfLifeState,
) -> AuthenticationResult<TokenState> {
    let access_token_cookie = cookies.get("wl_id").ok_or(AuthenticationError::Error(
        "Could not find access token.".to_owned(),
    ))?;

    let refresh_token_cookie = cookies.get("wl_rid").ok_or(AuthenticationError::Error(
        "Could not find refresh token.".to_owned(),
    ))?;

    Ok(verify_tokens(
        access_token_cookie.value(),
        refresh_token_cookie.value(),
        &state,
    )
    .await)
}
// TODO: Create a new error type for these `WebError/WebResult`
pub async fn user_info(
    cookies: Cookies,
    State(state): State<WaterOfLifeState>,
) -> AuthenticationResult<Response> {
    let is_token_valid = validate_cookies(cookies, &state).await?;

    let user_id = match is_token_valid {
        TokenState::Valid(user_id) => user_id,
        TokenState::RequiresRefresh(user_id, _user) => user_id,
        TokenState::Invalid => {
            return Ok(Redirect::to("/login").into_response())
        }
    };


    // TODO: Perform database lookup

    // tracing::debug!("user_info: {}", x.claims.sub);
    Ok("".into_response())
}

#[derive(Debug, Deserialize)]
struct SearchParameter {
    name: String,
}

pub fn search_spirit(
    cookies: Cookies,
    State(state): State<WaterOfLifeState>,
    Query(query_params): Query<SearchParameter>,
) -> AuthenticationResult<()> {
    todo!("search_spirit");
}

#[derive(Debug, Deserialize)]
struct SpiritPayload {
    name: String,
    distiller: String,
    description: String,
    abv: f64,
    image: Vec<u8>
}

pub fn add_spirit(
    cookies: Cookies,
    State(state): State<WaterOfLifeState>,
    Json(payload): Json<SpiritPayload>,
) -> AuthenticationResult<()> {
    todo!("add_spirit");
}

pub fn edit_spirit(
    cookies: Cookies,
    State(state): State<WaterOfLifeState>,
    Json(payload): Json<SpiritPayload>,
) -> AuthenticationResult<()> {
    todo!("edit_spirit");
}
