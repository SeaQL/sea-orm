use sea_orm::DbConn;

pub struct MigrationConnection<'c> {
    pub(crate) conn: &'c DbConn,
    pub(crate) schema_name: Option<String>,
}

impl<'c> From<&'c DbConn> for MigrationConnection<'c> {
    fn from(conn: &'c DbConn) -> Self {
        Self {
            conn,
            schema_name: None,
        }
    }
}

impl<'c> From<(&'c DbConn, String)> for MigrationConnection<'c> {
    fn from((conn, schema_name): (&'c DbConn, String)) -> Self {
        Self {
            conn,
            schema_name: Some(schema_name),
        }
    }
}
