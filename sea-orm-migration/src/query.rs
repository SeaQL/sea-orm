//! Get query result from db

use crate::into_migration_err;
use sea_schema::migration::{self, MigrationErr};

pub(crate) struct QueryResult {
    pub(crate) res: sea_orm::QueryResult,
}

impl migration::QueryResultTrait for QueryResult {
    fn try_get_string(&self, col: &str) -> Result<String, MigrationErr> {
        self.res
            .try_get::<String>("", col)
            .map_err(into_migration_err)
    }

    fn try_get_i64(&self, col: &str) -> Result<i64, MigrationErr> {
        self.res.try_get::<i64>("", col).map_err(into_migration_err)
    }
}
