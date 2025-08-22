use crate::rbac::{
    PermissionRequest, RbacEngine, RbacError, ResourceRequest, entity::user::UserId,
    schema::action_str,
};
use crate::{
    ConnectionTrait, DatabaseConnection, DbBackend, DbErr, ExecResult, QueryResult, Statement,
    StatementBuilder,
};
use std::sync::{Arc, RwLock};

/// Wrapper of [`DatabaseConnection`] that performs authorization on all executed
/// queries for the current user. Note that raw SQL [`Statement`] is not checked
/// currently.
#[derive(Debug, Clone)]
pub struct RestrictedConnection {
    pub(crate) user_id: UserId,
    pub(crate) conn: DatabaseConnection,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct RbacEngineHolder {
    inner: Arc<RwLock<Option<RbacEngine>>>,
}

impl RbacEngineHolder {
    pub fn is_some(&self) -> bool {
        let engine = self.inner.read().expect("RBAC Engine died");
        engine.is_some()
    }

    pub fn replace(&self, engine: RbacEngine) {
        let mut inner = self.inner.write().expect("RBAC Engine died");
        *inner = Some(engine);
    }

    pub fn user_can_run<S: StatementBuilder>(
        &self,
        user_id: UserId,
        stmt: &S,
    ) -> Result<(), DbErr> {
        let audit = match stmt.audit() {
            Ok(audit) => audit,
            Err(err) => return Err(DbErr::RbacError(err.to_string())),
        };
        for request in audit.requests {
            // There is nothing we can do if RwLock is poisoned.
            let holder = self.inner.read().expect("RBAC Engine died");
            // Constructor of this struct should ensure engine is not None.
            let engine = holder.as_ref().expect("RBAC Engine not set");
            let permission = || PermissionRequest {
                action: action_str(&request.access_type).to_owned(),
            };
            let resource = || ResourceRequest {
                schema: request.schema_table.0.as_ref().map(|s| s.1.to_string()),
                table: request.schema_table.1.to_string(),
            };
            if !engine
                .user_can(user_id, permission(), resource())
                .map_err(map_err)?
            {
                let r = resource();
                return Err(DbErr::AccessDenied {
                    permission: permission().action.to_owned(),
                    resource: format!(
                        "{}{}{}",
                        if let Some(schema) = &r.schema {
                            schema
                        } else {
                            ""
                        },
                        if r.schema.is_some() { "." } else { "" },
                        r.table
                    ),
                });
            }
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl ConnectionTrait for RestrictedConnection {
    fn get_database_backend(&self) -> DbBackend {
        self.conn.get_database_backend()
    }

    async fn execute_raw(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        Err(DbErr::RbacError(format!(
            "Raw query is not supported: {stmt}"
        )))
    }

    async fn execute<S: StatementBuilder>(&self, stmt: &S) -> Result<ExecResult, DbErr> {
        self.user_can_run(stmt)?;
        self.conn.execute(stmt).await
    }

    async fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
        self.conn.execute_unprepared(sql).await
    }

    async fn query_one_raw(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        Err(DbErr::RbacError(format!(
            "Raw query is not supported: {stmt}"
        )))
    }

    async fn query_one<S: StatementBuilder>(&self, stmt: &S) -> Result<Option<QueryResult>, DbErr> {
        self.user_can_run(stmt)?;
        self.conn.query_one(stmt).await
    }

    async fn query_all_raw(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        Err(DbErr::RbacError(format!(
            "Raw query is not supported: {stmt}"
        )))
    }

    async fn query_all<S: StatementBuilder>(&self, stmt: &S) -> Result<Vec<QueryResult>, DbErr> {
        self.user_can_run(stmt)?;
        self.conn.query_all(stmt).await
    }
}

impl RestrictedConnection {
    /// Get the [`RbacUserId`] bounded to this connection.
    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    /// Returns `()` if the current user can execute / query the given SQL statement.
    /// Returns `DbErr` otherwise.
    pub fn user_can_run<S: StatementBuilder>(&self, stmt: &S) -> Result<(), DbErr> {
        self.conn.rbac.user_can_run(self.user_id, stmt)
    }
}

fn map_err(err: RbacError) -> DbErr {
    DbErr::RbacError(err.to_string())
}
