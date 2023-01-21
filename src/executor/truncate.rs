use crate::{ConnectionTrait, DbBackend, DbErr, EntityTrait, Truncate};

impl<E> Truncate<E>
where
    E: EntityTrait,
{
    /// Perform truncate on the database:
    ///   - execute `TRUNCATE` on Postgres and MySQL
    ///   - execute `DELETE` on SQLite
    pub async fn exec<C>(self, db: &C) -> Result<(), DbErr>
    where
        C: ConnectionTrait,
    {
        let builder = db.get_database_backend();
        match builder {
            DbBackend::MySql | DbBackend::Postgres => {
                let stmt = builder.build(&self.query);
                db.execute(stmt).await?;
            }
            DbBackend::Sqlite => {
                E::delete_many().exec(db).await?;
            }
        }
        Ok(())
    }
}
