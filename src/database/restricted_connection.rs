use crate::rbac::{
    RbacError,
    engine::{PermissionRequest, RbacEngine, ResourceRequest},
    entity::user::UserId,
};
use crate::{
    ConnectionTrait, DatabaseConnection, DbBackend, DbErr, ExecResult, QueryResult, Statement,
    StatementBuilder,
    sea_query::audit::{AccessType, SchemaOper},
};
use std::sync::{Arc, RwLock};

/// Wrapper of [`DatabaseConnection`] that performs authorization on all executed
/// queries for the current user. Note that raw SQL [`Statement`] is not checked
/// currently.
#[derive(Debug)]
pub struct RestrictedConnection {
    user_id: UserId,
    conn: DatabaseConnection,
    engine: Arc<RwLock<RbacEngine>>,
}

#[async_trait::async_trait]
impl ConnectionTrait for RestrictedConnection {
    fn get_database_backend(&self) -> DbBackend {
        self.conn.get_database_backend()
    }

    async fn execute_raw(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        self.conn.execute_raw(stmt).await
    }

    async fn execute<S: StatementBuilder>(&self, stmt: &S) -> Result<ExecResult, DbErr> {
        self.user_can_run(stmt)?;
        self.conn.execute(stmt).await
    }

    async fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
        self.conn.execute_unprepared(sql).await
    }

    async fn query_one_raw(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        self.conn.query_one_raw(stmt).await
    }

    async fn query_one<S: StatementBuilder>(&self, stmt: &S) -> Result<Option<QueryResult>, DbErr> {
        self.user_can_run(stmt)?;
        self.conn.query_one(stmt).await
    }

    async fn query_all_raw(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        self.conn.query_all_raw(stmt).await
    }

    async fn query_all<S: StatementBuilder>(&self, stmt: &S) -> Result<Vec<QueryResult>, DbErr> {
        self.user_can_run(stmt)?;
        self.conn.query_all(stmt).await
    }
}

impl RestrictedConnection {
    /// Returns `()` if the current user can execute / query the given SQL statement.
    /// Returns `DbErr` otherwise.
    pub fn user_can_run<S: StatementBuilder>(&self, stmt: &S) -> Result<(), DbErr> {
        let audit = match stmt.audit() {
            Ok(audit) => audit,
            Err(err) => return Err(DbErr::RbacError(err.to_string())),
        };
        for request in audit.requests {
            let engine = self.engine.read().expect("RBAC Engine Died");
            let permission = || PermissionRequest {
                action: action(&request.access_type).to_owned(),
            };
            let resource = || ResourceRequest {
                schema: request.schema_table.0.as_ref().map(|s| s.to_string()),
                table: request.schema_table.1.to_string(),
            };
            if !engine
                .user_can(self.user_id, permission(), resource())
                .map_err(map_err)?
            {
                return Err(DbErr::AccessDenied {
                    permission: format!("{:?}", permission()),
                    resource: format!("{:?}", resource()),
                });
            }
        }
        Ok(())
    }
}

fn action(at: &AccessType) -> &'static str {
    match at {
        AccessType::Select => "select",
        AccessType::Insert => "insert",
        AccessType::Update => "update",
        AccessType::Delete => "delete",
        AccessType::Schema(SchemaOper::Create) => "schema_create",
        AccessType::Schema(SchemaOper::Alter) => "schema_alter",
        AccessType::Schema(SchemaOper::Drop) => "schema_drop",
        AccessType::Schema(SchemaOper::Rename) => "schema_rename",
        AccessType::Schema(SchemaOper::Truncate) => "schema_truncate",
    }
}

fn map_err(err: RbacError) -> DbErr {
    DbErr::RbacError(err.to_string())
}
