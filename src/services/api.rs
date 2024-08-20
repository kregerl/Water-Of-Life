use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    Extension, Json,
};
use futures::TryStreamExt;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use thiserror::Error;
use tower_cookies::Cookies;

use crate::{json_web::User, WaterOfLifeState};

#[derive(Error, Debug)]
pub enum WebError {
    #[error("Error quering database")]
    Database(#[from] sqlx::Error),
    #[error("Error serializing struct.")]
    Json(#[from] serde_json::Error),
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        let message = match self {
            WebError::Database(e) => e.to_string(),
            WebError::Json(e) => e.to_string(),
        };
        tracing::warn!("{}", message);
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

pub type WebResult<T> = Result<T, WebError>;

#[derive(Debug, Serialize)]
struct UserInfo {
    username: String,
    role: String,
    scopes: Vec<String>,
}

async fn get_scopes(pool: &SqlitePool, user_id: &str) -> sqlx::Result<Vec<String>> {
    let mut rows = sqlx::query_file!("sql/select_scopes.sql", user_id).fetch(pool);

    let mut scopes = Vec::new();
    while let Some(row) = rows.try_next().await? {
        tracing::info!("user_info: {}", row.scope);
        scopes.push(row.scope);
    }

    Ok(scopes)
}

pub async fn user_info(
    State(state): State<WaterOfLifeState>,
    Extension(user): Extension<User>,
) -> WebResult<Response> {
    let scopes = get_scopes(&state.database, &user.user_id).await?;

    let json = serde_json::to_string(&UserInfo {
        username: user.preferred_username,
        role: user.role,
        scopes,
    })?;

    Ok(json.into_response())
}

#[derive(Debug, Deserialize)]
struct SearchParameter {
    name: String,
}

pub fn search_spirit(
    cookies: Cookies,
    State(state): State<WaterOfLifeState>,
    Query(query_params): Query<SearchParameter>,
) -> WebResult<()> {
    todo!("search_spirit");
}

#[derive(Debug, Deserialize)]
struct SpiritPayload {
    name: String,
    distiller: String,
    description: String,
    abv: f64,
    image: Vec<u8>,
}

pub fn add_spirit(
    cookies: Cookies,
    State(state): State<WaterOfLifeState>,
    Json(payload): Json<SpiritPayload>,
) -> WebResult<()> {
    todo!("add_spirit");
}

pub fn edit_spirit(
    cookies: Cookies,
    State(state): State<WaterOfLifeState>,
    Json(payload): Json<SpiritPayload>,
) -> WebResult<()> {
    todo!("edit_spirit");
}
