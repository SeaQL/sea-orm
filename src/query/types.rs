use sqlx::mysql::MySqlRow;

pub struct QueryResult {
    pub(crate) row: QueryResultRow,
}

pub(crate) enum QueryResultRow {
    SqlxMySql(MySqlRow),
}
