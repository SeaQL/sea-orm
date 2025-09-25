use crate::rbac::{
    PermissionRequest, RbacEngine, RbacError, RbacPermissionsByResources,
    RbacResourcesAndPermissions, RbacRoleHierarchyList, RbacRolesAndRanks, RbacUserRolePermissions,
    ResourceRequest,
    entity::{role::RoleId, user::UserId},
};
use crate::{
    AccessMode, ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbBackend, DbErr,
    ExecResult, IsolationLevel, QueryResult, Statement, StatementBuilder, TransactionError,
    TransactionSession, TransactionTrait,
};
use std::{
    pin::Pin,
    sync::{Arc, RwLock},
};
use tracing::instrument;

/// Wrapper of [`DatabaseConnection`] that performs authorization on all executed
/// queries for the current user. Note that raw SQL [`Statement`] is not allowed
/// currently.
#[derive(Debug, Clone)]
#[cfg_attr(docsrs, doc(cfg(feature = "rbac")))]
pub struct RestrictedConnection {
    pub(crate) user_id: UserId,
    pub(crate) conn: DatabaseConnection,
}

/// Wrapper of [`DatabaseTransaction`] that performs authorization on all executed
/// queries for the current user. Note that raw SQL [`Statement`] is not allowed
/// currently.
#[derive(Debug)]
pub struct RestrictedTransaction {
    user_id: UserId,
    conn: DatabaseTransaction,
    rbac: RbacEngineMount,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct RbacEngineMount {
    inner: Arc<RwLock<Option<RbacEngine>>>,
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
        Err(DbErr::RbacError(format!(
            "Raw query is not supported: {sql}"
        )))
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

#[async_trait::async_trait]
impl ConnectionTrait for RestrictedTransaction {
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
        Err(DbErr::RbacError(format!(
            "Raw query is not supported: {sql}"
        )))
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

    /// Get current user's role and associated permissions.
    /// This includes permissions "inherited" from child roles.
    pub fn current_user_role_permissions(&self) -> Result<RbacUserRolePermissions, DbErr> {
        self.conn.rbac.user_role_permissions(self.user_id)
    }

    /// Get a list of all roles and their ranks.
    /// Rank is defined as (1 + number of child roles).
    pub fn roles_and_ranks(&self) -> Result<RbacRolesAndRanks, DbErr> {
        self.conn.rbac.roles_and_ranks()
    }

    /// Get two lists of all resources and permissions, excluding wildcards.
    pub fn resources_and_permissions(&self) -> Result<RbacResourcesAndPermissions, DbErr> {
        self.conn.rbac.resources_and_permissions()
    }

    /// Get a list of edges walking the role hierarchy tree
    pub fn role_hierarchy_edges(&self, role_id: RoleId) -> Result<RbacRoleHierarchyList, DbErr> {
        self.conn.rbac.role_hierarchy_edges(role_id)
    }

    /// Get a list of permissions for the specific role, grouped by resources.
    /// This does not include permissions of child roles.
    pub fn role_permissions_by_resources(
        &self,
        role_id: RoleId,
    ) -> Result<RbacPermissionsByResources, DbErr> {
        self.conn.rbac.role_permissions_by_resources(role_id)
    }
}

impl RestrictedTransaction {
    /// Get the [`RbacUserId`] bounded to this connection.
    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    /// Returns `()` if the current user can execute / query the given SQL statement.
    /// Returns `DbErr` otherwise.
    pub fn user_can_run<S: StatementBuilder>(&self, stmt: &S) -> Result<(), DbErr> {
        self.rbac.user_can_run(self.user_id, stmt)
    }
}

#[async_trait::async_trait]
impl TransactionTrait for RestrictedConnection {
    type Transaction = RestrictedTransaction;

    #[instrument(level = "trace")]
    async fn begin(&self) -> Result<RestrictedTransaction, DbErr> {
        Ok(RestrictedTransaction {
            user_id: self.user_id,
            conn: self.conn.begin().await?,
            rbac: self.conn.rbac.clone(),
        })
    }

    #[instrument(level = "trace")]
    async fn begin_with_config(
        &self,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<RestrictedTransaction, DbErr> {
        Ok(RestrictedTransaction {
            user_id: self.user_id,
            conn: self
                .conn
                .begin_with_config(isolation_level, access_mode)
                .await?,
            rbac: self.conn.rbac.clone(),
        })
    }

    /// Execute the function inside a transaction.
    /// If the function returns an error, the transaction will be rolled back. If it does not return an error, the transaction will be committed.
    #[instrument(level = "trace", skip(callback))]
    async fn transaction<F, T, E>(&self, callback: F) -> Result<T, TransactionError<E>>
    where
        F: for<'c> FnOnce(
                &'c RestrictedTransaction,
            ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'c>>
            + Send,
        T: Send,
        E: std::fmt::Display + std::fmt::Debug + Send,
    {
        let transaction = self.begin().await.map_err(TransactionError::Connection)?;
        transaction.run(callback).await
    }

    /// Execute the function inside a transaction.
    /// If the function returns an error, the transaction will be rolled back. If it does not return an error, the transaction will be committed.
    #[instrument(level = "trace", skip(callback))]
    async fn transaction_with_config<F, T, E>(
        &self,
        callback: F,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<T, TransactionError<E>>
    where
        F: for<'c> FnOnce(
                &'c RestrictedTransaction,
            ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'c>>
            + Send,
        T: Send,
        E: std::fmt::Display + std::fmt::Debug + Send,
    {
        let transaction = self
            .begin_with_config(isolation_level, access_mode)
            .await
            .map_err(TransactionError::Connection)?;
        transaction.run(callback).await
    }
}

#[async_trait::async_trait]
impl TransactionTrait for RestrictedTransaction {
    type Transaction = RestrictedTransaction;

    #[instrument(level = "trace")]
    async fn begin(&self) -> Result<RestrictedTransaction, DbErr> {
        Ok(RestrictedTransaction {
            user_id: self.user_id,
            conn: self.conn.begin().await?,
            rbac: self.rbac.clone(),
        })
    }

    #[instrument(level = "trace")]
    async fn begin_with_config(
        &self,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<RestrictedTransaction, DbErr> {
        Ok(RestrictedTransaction {
            user_id: self.user_id,
            conn: self
                .conn
                .begin_with_config(isolation_level, access_mode)
                .await?,
            rbac: self.rbac.clone(),
        })
    }

    /// Execute the function inside a transaction.
    /// If the function returns an error, the transaction will be rolled back. If it does not return an error, the transaction will be committed.
    #[instrument(level = "trace", skip(callback))]
    async fn transaction<F, T, E>(&self, callback: F) -> Result<T, TransactionError<E>>
    where
        F: for<'c> FnOnce(
                &'c RestrictedTransaction,
            ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'c>>
            + Send,
        T: Send,
        E: std::fmt::Display + std::fmt::Debug + Send,
    {
        let transaction = self.begin().await.map_err(TransactionError::Connection)?;
        transaction.run(callback).await
    }

    /// Execute the function inside a transaction.
    /// If the function returns an error, the transaction will be rolled back. If it does not return an error, the transaction will be committed.
    #[instrument(level = "trace", skip(callback))]
    async fn transaction_with_config<F, T, E>(
        &self,
        callback: F,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<T, TransactionError<E>>
    where
        F: for<'c> FnOnce(
                &'c RestrictedTransaction,
            ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'c>>
            + Send,
        T: Send,
        E: std::fmt::Display + std::fmt::Debug + Send,
    {
        let transaction = self
            .begin_with_config(isolation_level, access_mode)
            .await
            .map_err(TransactionError::Connection)?;
        transaction.run(callback).await
    }
}

#[async_trait::async_trait]
impl TransactionSession for RestrictedTransaction {
    async fn commit(self) -> Result<(), DbErr> {
        self.commit().await
    }

    async fn rollback(self) -> Result<(), DbErr> {
        self.rollback().await
    }
}

impl RestrictedTransaction {
    /// Runs a transaction to completion passing through the result.
    /// Rolling back the transaction on encountering an error.
    #[instrument(level = "trace", skip(callback))]
    async fn run<F, T, E>(self, callback: F) -> Result<T, TransactionError<E>>
    where
        F: for<'b> FnOnce(
                &'b RestrictedTransaction,
            ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'b>>
            + Send,
        T: Send,
        E: std::fmt::Display + std::fmt::Debug + Send,
    {
        let res = callback(&self).await.map_err(TransactionError::Transaction);
        if res.is_ok() {
            self.commit().await.map_err(TransactionError::Connection)?;
        } else {
            self.rollback()
                .await
                .map_err(TransactionError::Connection)?;
        }
        res
    }

    /// Commit a transaction
    #[instrument(level = "trace")]
    pub async fn commit(self) -> Result<(), DbErr> {
        self.conn.commit().await
    }

    /// Rolls back a transaction explicitly
    #[instrument(level = "trace")]
    pub async fn rollback(self) -> Result<(), DbErr> {
        self.conn.rollback().await
    }
}

impl RbacEngineMount {
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
                action: request.access_type.as_str().to_owned(),
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

    pub fn user_role_permissions(&self, user_id: UserId) -> Result<RbacUserRolePermissions, DbErr> {
        let holder = self.inner.read().expect("RBAC Engine died");
        let engine = holder.as_ref().expect("RBAC Engine not set");
        engine
            .get_user_role_permissions(user_id)
            .map_err(|err| DbErr::RbacError(err.to_string()))
    }

    pub fn roles_and_ranks(&self) -> Result<RbacRolesAndRanks, DbErr> {
        let holder = self.inner.read().expect("RBAC Engine died");
        let engine = holder.as_ref().expect("RBAC Engine not set");
        engine
            .get_roles_and_ranks()
            .map_err(|err| DbErr::RbacError(err.to_string()))
    }

    pub fn resources_and_permissions(&self) -> Result<RbacResourcesAndPermissions, DbErr> {
        let holder = self.inner.read().expect("RBAC Engine died");
        let engine = holder.as_ref().expect("RBAC Engine not set");
        Ok(engine.list_resources_and_permissions())
    }

    pub fn role_hierarchy_edges(&self, role_id: RoleId) -> Result<RbacRoleHierarchyList, DbErr> {
        let holder = self.inner.read().expect("RBAC Engine died");
        let engine = holder.as_ref().expect("RBAC Engine not set");
        Ok(engine.list_role_hierarchy_edges(role_id))
    }

    pub fn role_permissions_by_resources(
        &self,
        role_id: RoleId,
    ) -> Result<RbacPermissionsByResources, DbErr> {
        let holder = self.inner.read().expect("RBAC Engine died");
        let engine = holder.as_ref().expect("RBAC Engine not set");
        engine
            .list_role_permissions_by_resources(role_id)
            .map_err(|err| DbErr::RbacError(err.to_string()))
    }
}

fn map_err(err: RbacError) -> DbErr {
    DbErr::RbacError(err.to_string())
}
