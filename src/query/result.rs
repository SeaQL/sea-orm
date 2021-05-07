use sqlx::mysql::MySqlRow;

#[derive(Debug)]
pub struct QueryResult {
    pub(crate) row: QueryResultRow,
}

#[derive(Debug)]
pub(crate) enum QueryResultRow {
    SqlxMySql(MySqlRow),
}
