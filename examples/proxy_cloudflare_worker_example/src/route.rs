use anyhow::Result;
use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Router};
use worker::{console_error, console_log, Env};

use sea_orm::{
    ActiveModelTrait,
    ActiveValue::{NotSet, Set},
    EntityTrait,
};

#[derive(Clone)]
struct CFEnv {
    pub env: Arc<Env>,
}

unsafe impl Send for CFEnv {}
unsafe impl Sync for CFEnv {}

pub fn router(env: Env) -> Router {
    let state = CFEnv { env: Arc::new(env) };

    Router::new()
        .route("/", get(handler_get))
        .route("/generate", get(handler_generate))
        .with_state(state)
}

async fn handler_get(
    State(state): State<CFEnv>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let env = state.env.clone();
    let db = crate::orm::init_db(env).await.map_err(|err| {
        console_log!("Failed to connect to database: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to connect to database".to_string(),
        )
    })?;

    let ret = crate::entity::Entity::find()
        .all(&db)
        .await
        .map_err(|err| {
            console_log!("Failed to query database: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to query database".to_string(),
            )
        })?;
    let ret = serde_json::to_string(&ret).map_err(|err| {
        console_error!("Failed to serialize response: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to serialize response".to_string(),
        )
    })?;

    Ok(ret.into_response())
}

async fn handler_generate(
    State(state): State<CFEnv>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let env = state.env.clone();
    let db = crate::orm::init_db(env).await.map_err(|err| {
        console_log!("Failed to connect to database: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to connect to database".to_string(),
        )
    })?;

    let ret = crate::entity::ActiveModel {
        id: NotSet,
        title: Set(chrono::Utc::now().to_rfc3339()),
        text: Set(uuid::Uuid::new_v4().to_string()),
    };

    let ret = ret.insert(&db).await.map_err(|err| {
        console_log!("Failed to insert into database: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to insert into database".to_string(),
        )
    })?;

    Ok(format!("Inserted: {:?}", ret).into_response())
}
