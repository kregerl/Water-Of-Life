use std::{fs, io::Write};

use axum::{
    extract::{multipart::MultipartError, Multipart, Path, Query, State},
    response::{IntoResponse, Response},
    Extension, Form, Json,
};
use futures::TryStreamExt;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::{query, SqlitePool};
use thiserror::Error;
use tower_cookies::Cookies;
use uuid::Uuid;

use crate::{json_web::User, WaterOfLifeState};

pub const FORM_FILE_KEY: &'static str = "file";

#[derive(Error, Debug)]
pub enum WebError {
    #[error("Error quering database")]
    Database(#[from] sqlx::Error),
    #[error("Error serializing struct.")]
    Json(#[from] serde_json::Error),
    #[error("Error reading multipart request.")]
    MultipartError(#[from] MultipartError),
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        let mut status_code = StatusCode::INTERNAL_SERVER_ERROR;
        let message = match self {
            Self::Database(e) => e.to_string(),
            Self::Json(e) => e.to_string(),
            Self::MultipartError(e) => {
                status_code = StatusCode::BAD_REQUEST;
                e.to_string()
            }
        };
        tracing::warn!("{}", message);
        status_code.into_response()
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
    Extension(user): Extension<User>,
    State(state): State<WaterOfLifeState>,
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
pub struct SearchParameter {
    name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchResponse {
    uuid: Option<String>,
    name: Option<String>,
    distiller: Option<String>,
    bottler: Option<String>,
    typ: Option<String>,
}

pub async fn search_spirit(
    State(state): State<WaterOfLifeState>,
    Query(query_params): Query<SearchParameter>,
) -> WebResult<Response> {
    let names = sqlx::query_file_as!(SearchResponse, "sql/search_spirit.sql", query_params.name)
        .fetch_all(&state.database)
        .await?;

    let response = serde_json::to_string(&names)?;
    Ok(response.into_response())
}

#[derive(Debug, Deserialize)]
pub struct SpiritPayload {
    name: String,
    distiller: String,
    description: String,
    abv: f64,
}

#[derive(Debug, Serialize)]
pub struct SpiritResponse {
    id: String,
}

pub async fn add_spirit(
    State(state): State<WaterOfLifeState>,
    Json(payload): Json<SpiritPayload>,
) -> WebResult<Response> {
    tracing::debug!("add_spirit: {:#?}", payload.name);
    tracing::debug!("add_spirit: {:#?}", payload.distiller);
    tracing::debug!("add_spirit: {:#?}", payload.description);
    tracing::debug!("add_spirit: {:#?}", payload.abv);

    let id = Uuid::new_v4().to_string();
    let _ = sqlx::query_file!(
        "sql/insert_spirit.sql",
        id,
        payload.name,
        payload.distiller,
        payload.description,
        payload.abv
    )
    .execute(&state.database)
    .await?;

    let response = serde_json::to_string(&SpiritResponse { id })?;
    Ok(response.into_response())
}

pub async fn upload_spirit_image(
    Extension(user): Extension<User>,
    State(state): State<WaterOfLifeState>,
    Path(spirit_id): Path<String>,
    mut multipart: Multipart,
) -> WebResult<Response> {
    tracing::debug!("upload_spirit_image: Got spirit id: {}", spirit_id);
    while let Some(field) = multipart.next_field().await? {
        let name = if let Some(name) = field.name() {
            name.to_owned()
        } else {
            continue;
        };

        if name != FORM_FILE_KEY {
            continue;
        }

        let data = field.bytes().await?;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(state.images_path.join(&spirit_id))
            .unwrap();

        file.write_all(&data).unwrap();
        tracing::debug!("Length of `{}` is {} bytes", name, data.len());
    }

    Ok("".into_response())
}

pub async fn get_spirit_image(
    State(state): State<WaterOfLifeState>,
    Path(spirit_id): Path<String>,
) -> WebResult<Response> {
    Ok("".into_response())
}

pub async fn edit_spirit(
    cookies: Cookies,
    State(state): State<WaterOfLifeState>,
    Json(payload): Json<SpiritPayload>,
) -> WebResult<()> {
    todo!("edit_spirit");
}
