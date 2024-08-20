use axum::{
    extract::{MatchedPath, Request, State},
    middleware::Next,
    response::Response,
};
use reqwest::StatusCode;
use tower_cookies::{
    cookie::{time::Duration, SameSite},
    Cookie, Cookies,
};
use tower_sessions::{MemoryStore, SessionManagerLayer};

use crate::{
    json_web::{self, generate_access_and_refresh_tokens, verify_tokens, TokenState},
    WaterOfLifeState,
};

pub fn create_span(request: &Request) -> tracing::Span {
    let method = request.method();
    let uri = request.uri();

    let matched_path = request
        .extensions()
        .get::<MatchedPath>()
        .map(|matched_path| matched_path.as_str())
        .unwrap_or("<unknown>");

    tracing::debug_span!("request", %method, %uri, matched_path)
}

pub fn session_layer() -> SessionManagerLayer<MemoryStore> {
    // Probably fine to store nonces in memory for now since theyre 32 bytes each
    SessionManagerLayer::new(MemoryStore::default())
        .with_same_site(SameSite::Lax)
        // FIXME: This should be removed once the web server is running HTTPS
        .with_secure(false)
        .with_expiry(tower_sessions::Expiry::OnInactivity(Duration::minutes(2)))
}

#[allow(clippy::unused_async)]
pub async fn handle_error() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "That endpoint does not exist.")
}

async fn validate_cookies(
    cookies: &Cookies,
    state: &WaterOfLifeState,
) -> Result<TokenState, String> {
    let access_token_cookie = cookies
        .get("wl_id")
        .ok_or("Could not find access token.".to_owned())?;

    let refresh_token_cookie = cookies
        .get("wl_rid")
        .ok_or("Could not find refresh token.".to_owned())?;

    Ok(verify_tokens(
        access_token_cookie.value(),
        refresh_token_cookie.value(),
        &state,
    )
    .await)
}

fn create_token_cookie<'a>(key: &'a str, token: String) -> Cookie<'a> {
    let mut cookie = Cookie::new(key, token);
    cookie.set_path("/");
    cookie.set_secure(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_http_only(true);
    cookie
}

pub async fn authentication(
    State(state): State<WaterOfLifeState>,
    cookies: Cookies,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if !request.uri().path().starts_with("/api") {
        return Ok(next.run(request).await);
    }

    let is_token_valid = match validate_cookies(&cookies, &state).await {
        Ok(is_token_valid) => is_token_valid,
        Err(e) => {
            tracing::debug!("{}", e);
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    let user_id = match is_token_valid {
        TokenState::Valid(user_id) => user_id,
        TokenState::RequiresRefresh(user_id, user) => {
            if let Some((access_token, refresh_token)) = generate_access_and_refresh_tokens(
                &state.access_token_hmac_secret,
                &state.refresh_token_hmac_secret,
                &state.client_id,
                &user_id,
                &user.role,
            ) {
                cookies.add(create_token_cookie("wl_id", access_token));
                cookies.add(create_token_cookie("wl_rid", refresh_token));
            }
            user_id
        }
        TokenState::Invalid => return Err(StatusCode::UNAUTHORIZED),
    };

    let user = sqlx::query_file_as!(json_web::User, "sql/select_user.sql", user_id)
        .fetch_one(&state.database)
        .await
        .unwrap();

    tracing::info!("Got user: {:#?}", user);
    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}
