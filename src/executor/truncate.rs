use crate::{ConnectionTrait, DbBackend, DbErr, DeleteResult, EntityTrait, ExecResult, Truncate};

/// Truncate result
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TruncateResult {
    /// Number of database rows being affected
    pub rows_affected: u64,
}

impl<E> Truncate<E>
where
    E: EntityTrait,
{
    /// Perform truncate on the database:
    ///   - execute `TRUNCATE` on Postgres and MySQL
    ///   - execute `DELETE` on SQLite
    pub async fn exec<C>(self, db: &C) -> Result<TruncateResult, DbErr>
    where
        C: ConnectionTrait,
    {
        let builder = db.get_database_backend();
        match builder {
            DbBackend::MySql | DbBackend::Postgres => {
                let stmt = builder.build(&self.query);
                db.execute(stmt).await.map(Into::into)
            }
            DbBackend::Sqlite => E::delete_many().exec(db).await.map(Into::into),
        }
    }
}

impl From<ExecResult> for TruncateResult {
    fn from(res: ExecResult) -> TruncateResult {
        TruncateResult {
            rows_affected: res.rows_affected(),
        }
    }
}

impl From<DeleteResult> for TruncateResult {
    fn from(res: DeleteResult) -> TruncateResult {
        TruncateResult {
            rows_affected: res.rows_affected,
        }
    }
}
