use crate::{DatabaseConnection, DbBackend, Statement};
use sea_query::QueryStatementBuilder;

pub trait QueryTrait {
    type QueryStatement: QueryStatementBuilder;

    /// Get a mutable ref to the query builder
    fn query(&mut self) -> &mut Self::QueryStatement;

    /// Get an immutable ref to the query builder
    fn as_query(&self) -> &Self::QueryStatement;

    /// Take ownership of the query builder
    fn into_query(self) -> Self::QueryStatement;

    /// Build the query as [`Statement`]
    fn build(&self, db_backend: DbBackend) -> Statement {
        let query_builder = db_backend.get_query_builder();
        Statement::from_string_values_tuple(
            db_backend,
            self.as_query().build_any(query_builder.as_ref()),
        )
    }
}

#[derive(Debug)]
pub struct DebugQuery<'a, Q, T> {
    pub query: &'a Q,
    pub value: T,
}

impl<'a, Q> DebugQuery<'a, Q, DbBackend>
where
    Q: QueryTrait,
{
    pub fn build(&self) -> Statement {
        self.query.build(self.value)
    }
}

impl<'a, Q> DebugQuery<'a, Q, &DatabaseConnection>
where
    Q: QueryTrait,
{
    pub fn build(&self) -> Statement {
        self.query.build(self.value.get_database_backend())
    }
}

/// Make get raw_sql becomes simply. It does not need to specify a specific `DbBackend`,
/// but can be obtained through `get_database_backend` with `DatabaseConnection`.
/// Return a Statement type.
///
///
/// # Example
///
/// ```
/// # #[cfg(feature = "mock")]
/// # use sea_orm::{error::*, tests_cfg::*, MockDatabase, MockExecResult, Transaction, DbBackend};
/// #
/// # let conn = MockDatabase::new(DbBackend::Postgres)
/// #     .into_connection();
/// #
/// use sea_orm::{entity::*, query::*, tests_cfg::cake,debug_query};
///
/// let c = cake::Entity::insert(
///    cake::ActiveModel {
///         id: ActiveValue::set(1),
///         name: ActiveValue::set("Apple Pie".to_owned()),
/// });
///
/// let raw_sql = debug_query!(&c,&conn).to_string();
/// assert_eq!(raw_sql,r#"INSERT INTO "cake" ("id", "name") VALUES (1, 'Apple Pie')"#);
///
/// let raw_sql = debug_query!(&c,DbBackend::MySql).to_string();
/// assert_eq!(raw_sql,r#"INSERT INTO `cake` (`id`, `name`) VALUES (1, 'Apple Pie')"#);
///
/// ```
#[macro_export]
macro_rules! debug_query {
    ($query:expr,$value:expr) => {
        $crate::DebugQuery {
            query: $query,
            value: $value,
        }
        .build();
    };
}

/// Use `debug_query` macro get raw_sql.
///
/// # Example
///
/// ```
/// # #[cfg(feature = "mock")]
/// # use sea_orm::{error::*, tests_cfg::*, MockDatabase, MockExecResult, Transaction, DbBackend};
/// #
/// # let conn = MockDatabase::new(DbBackend::Postgres)
/// #     .into_connection();
/// #
/// use sea_orm::{entity::*, query::*, tests_cfg::cake,debug_query_fmt};
///
/// let c = cake::Entity::insert(
///    cake::ActiveModel {
///         id: ActiveValue::set(1),
///         name: ActiveValue::set("Apple Pie".to_owned()),
/// });
///
/// let raw_sql = debug_query_fmt!(&c,&conn);
/// assert_eq!(raw_sql,r#"INSERT INTO "cake" ("id", "name") VALUES (1, 'Apple Pie')"#);
///
/// let raw_sql = debug_query_fmt!(&c,DbBackend::Sqlite);
/// assert_eq!(raw_sql,r#"INSERT INTO `cake` (`id`, `name`) VALUES (1, 'Apple Pie')"#);
///
/// ```
#[macro_export]
macro_rules! debug_query_fmt {
    ($query:expr,$value:expr) => {
        $crate::debug_query!($query, $value).to_string();
    };
}
