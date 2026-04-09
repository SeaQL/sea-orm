use axum::http::StatusCode;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, Schema};
use worker::console_error;

pub async fn ensure_schema(db: &DatabaseConnection) -> Result<(), DbErr> {
    let backend = db.get_database_backend();
    db.execute(
        backend.build(
            Schema::new(backend)
                .create_table_from_entity(crate::entity::Entity)
                .if_not_exists(),
        ),
    )
    .await?;

    Ok(())
}

// If you are learning from examples, this is not a recommended way of handling it,
// it was used here only for simplicity and to preserve remnants of previous code.
// Instead, it is recommended to use the `IntoResponse` trait.
pub fn map_error<E: std::fmt::Debug>(err: E, msg: &str) -> (StatusCode, String) {
    console_error!("{}: {:?}", msg, err);
    (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string())
}
