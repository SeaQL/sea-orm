use std::sync::Arc;

use axum::{Router, extract::State, http::StatusCode, response::IntoResponse, routing::get};

use sea_orm::{
    ActiveModelTrait,
    ActiveValue::{NotSet, Set},
    DatabaseConnection, EntityTrait,
};

use crate::utility::map_error;

struct AppState {
    pub db: DatabaseConnection,
}

pub fn router(db: DatabaseConnection) -> Router {
    // generally, it is much simpler and cleaner to wrap the AppState itself,
    // rather than its individual members.
    let state = Arc::new(AppState { db });

    Router::new()
        .route("/", get(handler_get))
        .route("/generate", get(handler_generate))
        .with_state(state)
}

async fn handler_get(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    crate::utility::ensure_schema(&state.db)
        .await
        .map_err(|err| map_error(err, "Failed to create table"))?;

    let ret = crate::entity::Entity::find()
        .all(&state.db)
        .await
        .map_err(|err| map_error(err, "Failed to query database"))?;
    let ret = serde_json::to_string(&ret)
        .map_err(|err| map_error(err, "Failed to serialize response"))?;

    Ok(ret.into_response())
}

async fn handler_generate(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    crate::utility::ensure_schema(&state.db)
        .await
        .map_err(|err| map_error(err, "Failed to serialize response"))?;

    let ret = crate::entity::ActiveModel {
        id: NotSet,
        title: Set(chrono::Utc::now().to_rfc3339()),
        text: Set(uuid::Uuid::new_v4().to_string()),
    };

    let ret = ret
        .insert(&state.db)
        .await
        .map_err(|err| map_error(err, "Failed to insert into database"))?;

    Ok(format!("Inserted: {:?}", ret).into_response())
}
