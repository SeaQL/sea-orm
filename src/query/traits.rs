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

macro_rules! debug_query_build {
    ($impl_obj:ty,$db_expr:tt) => {
        impl<'a, Q> DebugQuery<'a, Q, $impl_obj>
        where
            Q: QueryTrait,
        {
            pub fn build(&self) -> Statement {
                let db_backend = $db_expr(self);
                self.query.build(db_backend)
            }
        }
    };
}

debug_query_build!(DbBackend, (|x: &DebugQuery<_, DbBackend>| x.value));
debug_query_build!(&DbBackend, (|x: &DebugQuery<_, &DbBackend>| *x.value));
debug_query_build!(
    DatabaseConnection,
    (|x: &DebugQuery<_, DatabaseConnection>| x.value.get_database_backend())
);
debug_query_build!(
    &DatabaseConnection,
    (|x: &DebugQuery<_, &DatabaseConnection>| x.value.get_database_backend())
);

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
/// use sea_orm::{entity::*, query::*, tests_cfg::cake,gen_statement};
///
/// let c = cake::Entity::insert(
///    cake::ActiveModel {
///         id: ActiveValue::set(1),
///         name: ActiveValue::set("Apple Pie".to_owned()),
/// });
///
/// let raw_sql = gen_statement!(&c,&conn).to_string();
/// assert_eq!(raw_sql,r#"INSERT INTO "cake" ("id", "name") VALUES (1, 'Apple Pie')"#);
///
/// let raw_sql = gen_statement!(&c,conn).to_string();
/// assert_eq!(raw_sql,r#"INSERT INTO "cake" ("id", "name") VALUES (1, 'Apple Pie')"#);
///
/// let raw_sql = gen_statement!(&c,DbBackend::MySql).to_string();
/// assert_eq!(raw_sql,r#"INSERT INTO `cake` (`id`, `name`) VALUES (1, 'Apple Pie')"#);
///
/// let raw_sql = gen_statement!(&c,&DbBackend::MySql).to_string();
/// assert_eq!(raw_sql,r#"INSERT INTO `cake` (`id`, `name`) VALUES (1, 'Apple Pie')"#);
///
/// ```
#[macro_export]
macro_rules! gen_statement {
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
/// use sea_orm::{entity::*, query::*, tests_cfg::cake,debug_query};
///
/// let c = cake::Entity::insert(
///    cake::ActiveModel {
///         id: ActiveValue::set(1),
///         name: ActiveValue::set("Apple Pie".to_owned()),
/// });
///
/// let raw_sql = debug_query!(&c,&conn);
/// assert_eq!(raw_sql,r#"INSERT INTO "cake" ("id", "name") VALUES (1, 'Apple Pie')"#);
///
/// let raw_sql = debug_query!(&c,conn);
/// assert_eq!(raw_sql,r#"INSERT INTO "cake" ("id", "name") VALUES (1, 'Apple Pie')"#);
///
/// let raw_sql = debug_query!(&c,DbBackend::Sqlite);
/// assert_eq!(raw_sql,r#"INSERT INTO `cake` (`id`, `name`) VALUES (1, 'Apple Pie')"#);
///
/// ```
#[macro_export]
macro_rules! debug_query {
    ($query:expr,$value:expr) => {
        $crate::gen_statement!($query, $value).to_string();
    };
}
