#[cfg(feature = "mock")]
use crate::debug_print;
use crate::{DbErr, SelectGetableValue, SelectorRaw, Statement};
use std::fmt;

/// Defines the result of a query operation on a Model
#[derive(Debug)]
pub struct QueryResult {
    pub(crate) row: QueryResultRow,
}

#[allow(clippy::enum_variant_names)]
pub(crate) enum QueryResultRow {
    #[cfg(feature = "sqlx-mysql")]
    SqlxMySql(sqlx::mysql::MySqlRow),
    #[cfg(feature = "sqlx-postgres")]
    SqlxPostgres(sqlx::postgres::PgRow),
    #[cfg(feature = "sqlx-sqlite")]
    SqlxSqlite(sqlx::sqlite::SqliteRow),
    #[cfg(feature = "mock")]
    Mock(crate::MockRow),
}

/// Constrain any type trying to get a Row in a database
pub trait TryGetable: Sized {
    /// Ensure the type implements this method
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError>;
}

/// An error from trying to get a row from a Model
#[derive(Debug)]
pub enum TryGetError {
    /// A database error was encountered as defined in [crate::DbErr]
    DbErr(DbErr),
    /// A null value was encountered
    Null,
}

impl From<TryGetError> for DbErr {
    fn from(e: TryGetError) -> DbErr {
        match e {
            TryGetError::DbErr(e) => e,
            TryGetError::Null => DbErr::Query("error occurred while decoding: Null".to_owned()),
        }
    }
}

// QueryResult //

impl QueryResult {
    /// Get a Row from a Column
    pub fn try_get<T>(&self, pre: &str, col: &str) -> Result<T, DbErr>
    where
        T: TryGetable,
    {
        Ok(T::try_get(self, pre, col)?)
    }

    /// Perform query operations on multiple Columns
    pub fn try_get_many<T>(&self, pre: &str, cols: &[String]) -> Result<T, DbErr>
    where
        T: TryGetableMany,
    {
        Ok(T::try_get_many(self, pre, cols)?)
    }
}

impl fmt::Debug for QueryResultRow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            Self::SqlxMySql(row) => write!(f, "{:?}", row),
            #[cfg(feature = "sqlx-postgres")]
            Self::SqlxPostgres(_) => write!(f, "QueryResultRow::SqlxPostgres cannot be inspected"),
            #[cfg(feature = "sqlx-sqlite")]
            Self::SqlxSqlite(_) => write!(f, "QueryResultRow::SqlxSqlite cannot be inspected"),
            #[cfg(feature = "mock")]
            Self::Mock(row) => write!(f, "{:?}", row),
            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        }
    }
}

// TryGetable //

impl<T: TryGetable> TryGetable for Option<T> {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        match T::try_get(res, pre, col) {
            Ok(v) => Ok(Some(v)),
            Err(TryGetError::Null) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

macro_rules! try_getable_all {
    ( $type: ty ) => {
        impl TryGetable for $type {
            fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
                let column = format!("{}{}", pre, col);
                match &res.row {
                    #[cfg(feature = "sqlx-mysql")]
                    QueryResultRow::SqlxMySql(row) => {
                        use sqlx::Row;
                        row.try_get::<Option<$type>, _>(column.as_str())
                            .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                            .and_then(|opt| opt.ok_or(TryGetError::Null))
                    }
                    #[cfg(feature = "sqlx-postgres")]
                    QueryResultRow::SqlxPostgres(row) => {
                        use sqlx::Row;
                        row.try_get::<Option<$type>, _>(column.as_str())
                            .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                            .and_then(|opt| opt.ok_or(TryGetError::Null))
                    }
                    #[cfg(feature = "sqlx-sqlite")]
                    QueryResultRow::SqlxSqlite(row) => {
                        use sqlx::Row;
                        row.try_get::<Option<$type>, _>(column.as_str())
                            .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                            .and_then(|opt| opt.ok_or(TryGetError::Null))
                    }
                    #[cfg(feature = "mock")]
                    #[allow(unused_variables)]
                    QueryResultRow::Mock(row) => row.try_get(column.as_str()).map_err(|e| {
                        debug_print!("{:#?}", e.to_string());
                        TryGetError::Null
                    }),
                    #[allow(unreachable_patterns)]
                    _ => unreachable!(),
                }
            }
        }
    };
}

macro_rules! try_getable_unsigned {
    ( $type: ty ) => {
        impl TryGetable for $type {
            fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
                let _column = format!("{}{}", pre, col);
                match &res.row {
                    #[cfg(feature = "sqlx-mysql")]
                    QueryResultRow::SqlxMySql(row) => {
                        use sqlx::Row;
                        row.try_get::<Option<$type>, _>(_column.as_str())
                            .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                            .and_then(|opt| opt.ok_or(TryGetError::Null))
                    }
                    #[cfg(feature = "sqlx-postgres")]
                    QueryResultRow::SqlxPostgres(_) => {
                        panic!("{} unsupported by sqlx-postgres", stringify!($type))
                    }
                    #[cfg(feature = "sqlx-sqlite")]
                    QueryResultRow::SqlxSqlite(row) => {
                        use sqlx::Row;
                        row.try_get::<Option<$type>, _>(_column.as_str())
                            .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                            .and_then(|opt| opt.ok_or(TryGetError::Null))
                    }
                    #[cfg(feature = "mock")]
                    #[allow(unused_variables)]
                    QueryResultRow::Mock(row) => row.try_get(_column.as_str()).map_err(|e| {
                        debug_print!("{:#?}", e.to_string());
                        TryGetError::Null
                    }),
                    #[allow(unreachable_patterns)]
                    _ => unreachable!(),
                }
            }
        }
    };
}

macro_rules! try_getable_mysql {
    ( $type: ty ) => {
        impl TryGetable for $type {
            fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
                let _column = format!("{}{}", pre, col);
                match &res.row {
                    #[cfg(feature = "sqlx-mysql")]
                    QueryResultRow::SqlxMySql(row) => {
                        use sqlx::Row;
                        row.try_get::<Option<$type>, _>(_column.as_str())
                            .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                            .and_then(|opt| opt.ok_or(TryGetError::Null))
                    }
                    #[cfg(feature = "sqlx-postgres")]
                    QueryResultRow::SqlxPostgres(_) => {
                        panic!("{} unsupported by sqlx-postgres", stringify!($type))
                    }
                    #[cfg(feature = "sqlx-sqlite")]
                    QueryResultRow::SqlxSqlite(_) => {
                        panic!("{} unsupported by sqlx-sqlite", stringify!($type))
                    }
                    #[cfg(feature = "mock")]
                    #[allow(unused_variables)]
                    QueryResultRow::Mock(row) => row.try_get(_column.as_str()).map_err(|e| {
                        debug_print!("{:#?}", e.to_string());
                        TryGetError::Null
                    }),
                    #[allow(unreachable_patterns)]
                    _ => unreachable!(),
                }
            }
        }
    };
}

macro_rules! try_getable_date_time {
    ( $type: ty ) => {
        impl TryGetable for $type {
            fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
                let _column = format!("{}{}", pre, col);
                match &res.row {
                    #[cfg(feature = "sqlx-mysql")]
                    QueryResultRow::SqlxMySql(row) => {
                        use chrono::{DateTime, Utc};
                        use sqlx::Row;
                        row.try_get::<Option<DateTime<Utc>>, _>(_column.as_str())
                            .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                            .and_then(|opt| opt.ok_or(TryGetError::Null))
                            .map(|v| v.into())
                    }
                    #[cfg(feature = "sqlx-postgres")]
                    QueryResultRow::SqlxPostgres(row) => {
                        use sqlx::Row;
                        row.try_get::<Option<$type>, _>(_column.as_str())
                            .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                            .and_then(|opt| opt.ok_or(TryGetError::Null))
                    }
                    #[cfg(feature = "sqlx-sqlite")]
                    QueryResultRow::SqlxSqlite(row) => {
                        use chrono::{DateTime, Utc};
                        use sqlx::Row;
                        row.try_get::<Option<DateTime<Utc>>, _>(_column.as_str())
                            .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                            .and_then(|opt| opt.ok_or(TryGetError::Null))
                            .map(|v| v.into())
                    }
                    #[cfg(feature = "mock")]
                    #[allow(unused_variables)]
                    QueryResultRow::Mock(row) => row.try_get(_column.as_str()).map_err(|e| {
                        debug_print!("{:#?}", e.to_string());
                        TryGetError::Null
                    }),
                    #[allow(unreachable_patterns)]
                    _ => unreachable!(),
                }
            }
        }
    };
}

macro_rules! try_getable_time {
    ( $type: ty ) => {
        impl TryGetable for $type {
            fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
                let column = format!("{}{}", pre, col);
                match &res.row {
                    #[cfg(feature = "sqlx-mysql")]
                    QueryResultRow::SqlxMySql(row) => {
                        use sqlx::Row;
                        row.try_get::<Option<$type>, _>(column.as_str())
                            .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                            .and_then(|opt| opt.ok_or(TryGetError::Null))
                    }
                    #[cfg(feature = "sqlx-postgres")]
                    QueryResultRow::SqlxPostgres(row) => {
                        use sqlx::Row;
                        row.try_get::<Option<$type>, _>(column.as_str())
                            .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                            .and_then(|opt| opt.ok_or(TryGetError::Null))
                    }
                    #[cfg(feature = "sqlx-sqlite")]
                    QueryResultRow::SqlxSqlite(_) => {
                        panic!("{} unsupported by sqlx-sqlite", stringify!($type))
                    }
                    #[cfg(feature = "mock")]
                    #[allow(unused_variables)]
                    QueryResultRow::Mock(row) => row.try_get(column.as_str()).map_err(|e| {
                        debug_print!("{:#?}", e.to_string());
                        TryGetError::Null
                    }),
                    #[allow(unreachable_patterns)]
                    _ => unreachable!(),
                }
            }
        }
    };
}

try_getable_all!(bool);
try_getable_all!(i8);
try_getable_all!(i16);
try_getable_all!(i32);
try_getable_all!(i64);
try_getable_unsigned!(u8);
try_getable_unsigned!(u16);
try_getable_mysql!(u64);
try_getable_all!(f32);
try_getable_all!(f64);
try_getable_all!(String);
try_getable_all!(Vec<u8>);

#[cfg(feature = "with-json")]
try_getable_all!(serde_json::Value);

#[cfg(feature = "with-chrono")]
try_getable_all!(chrono::NaiveDate);

#[cfg(feature = "with-chrono")]
try_getable_all!(chrono::NaiveTime);

#[cfg(feature = "with-chrono")]
try_getable_all!(chrono::NaiveDateTime);

#[cfg(feature = "with-chrono")]
try_getable_date_time!(chrono::DateTime<chrono::FixedOffset>);

#[cfg(feature = "with-chrono")]
try_getable_all!(chrono::DateTime<chrono::Utc>);

#[cfg(feature = "with-chrono")]
try_getable_all!(chrono::DateTime<chrono::Local>);

#[cfg(feature = "with-time")]
try_getable_time!(time::Date);

#[cfg(feature = "with-time")]
try_getable_time!(time::Time);

#[cfg(feature = "with-time")]
try_getable_time!(time::PrimitiveDateTime);

#[cfg(feature = "with-time")]
try_getable_time!(time::OffsetDateTime);

#[cfg(feature = "with-rust_decimal")]
use rust_decimal::Decimal;

#[cfg(feature = "with-rust_decimal")]
impl TryGetable for Decimal {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let column = format!("{}{}", pre, col);
        match &res.row {
            #[cfg(feature = "sqlx-mysql")]
            QueryResultRow::SqlxMySql(row) => {
                use sqlx::Row;
                row.try_get::<Option<Decimal>, _>(column.as_str())
                    .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                    .and_then(|opt| opt.ok_or(TryGetError::Null))
            }
            #[cfg(feature = "sqlx-postgres")]
            QueryResultRow::SqlxPostgres(row) => {
                use sqlx::Row;
                row.try_get::<Option<Decimal>, _>(column.as_str())
                    .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                    .and_then(|opt| opt.ok_or(TryGetError::Null))
            }
            #[cfg(feature = "sqlx-sqlite")]
            QueryResultRow::SqlxSqlite(row) => {
                use sqlx::Row;
                let val: Option<f64> = row
                    .try_get(column.as_str())
                    .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))?;
                use rust_decimal::prelude::FromPrimitive;
                match val {
                    Some(v) => Decimal::from_f64(v).ok_or_else(|| {
                        TryGetError::DbErr(DbErr::Query(
                            "Failed to convert f64 into Decimal".to_owned(),
                        ))
                    }),
                    None => Err(TryGetError::Null),
                }
            }
            #[cfg(feature = "mock")]
            #[allow(unused_variables)]
            QueryResultRow::Mock(row) => row.try_get(column.as_str()).map_err(|e| {
                debug_print!("{:#?}", e.to_string());
                TryGetError::Null
            }),
            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        }
    }
}

#[cfg(feature = "with-uuid")]
try_getable_all!(uuid::Uuid);

impl TryGetable for u32 {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let _column = format!("{}{}", pre, col);
        match &res.row {
            #[cfg(feature = "sqlx-mysql")]
            QueryResultRow::SqlxMySql(row) => {
                use sqlx::Row;
                row.try_get::<Option<u32>, _>(_column.as_str())
                    .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                    .and_then(|opt| opt.ok_or(TryGetError::Null))
            }
            #[cfg(feature = "sqlx-postgres")]
            QueryResultRow::SqlxPostgres(row) => {
                use sqlx::postgres::types::Oid;
                // Since 0.6.0, SQLx has dropped direct mapping from PostgreSQL's OID to Rust's `u32`;
                // Instead, `u32` was wrapped by a `sqlx::Oid`.
                use sqlx::Row;
                row.try_get::<Option<Oid>, _>(_column.as_str())
                    .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                    .and_then(|opt| opt.ok_or(TryGetError::Null))
                    .map(|oid| oid.0)
            }
            #[cfg(feature = "sqlx-sqlite")]
            QueryResultRow::SqlxSqlite(row) => {
                use sqlx::Row;
                row.try_get::<Option<u32>, _>(_column.as_str())
                    .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                    .and_then(|opt| opt.ok_or(TryGetError::Null))
            }
            #[cfg(feature = "mock")]
            #[allow(unused_variables)]
            QueryResultRow::Mock(row) => row.try_get(_column.as_str()).map_err(|e| {
                debug_print!("{:#?}", e.to_string());
                TryGetError::Null
            }),
            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        }
    }
}

// TryGetableMany //

/// Perform a query on multiple columns
pub trait TryGetableMany: Sized {
    /// THe method to perform a query on multiple columns
    fn try_get_many(res: &QueryResult, pre: &str, cols: &[String]) -> Result<Self, TryGetError>;

    /// ```
    /// # use sea_orm::{error::*, tests_cfg::*, *};
    /// #
    /// # #[smol_potat::main]
    /// # #[cfg(all(feature = "mock", feature = "macros"))]
    /// # pub async fn main() -> Result<(), DbErr> {
    /// #
    /// # let db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_query_results(vec![vec![
    /// #         maplit::btreemap! {
    /// #             "name" => Into::<Value>::into("Chocolate Forest"),
    /// #             "num_of_cakes" => Into::<Value>::into(1),
    /// #         },
    /// #         maplit::btreemap! {
    /// #             "name" => Into::<Value>::into("New York Cheese"),
    /// #             "num_of_cakes" => Into::<Value>::into(1),
    /// #         },
    /// #     ]])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DeriveIden, EnumIter, TryGetableMany};
    ///
    /// #[derive(EnumIter, DeriveIden)]
    /// enum ResultCol {
    ///     Name,
    ///     NumOfCakes,
    /// }
    ///
    /// let res: Vec<(String, i32)> =
    ///     <(String, i32)>::find_by_statement::<ResultCol>(Statement::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"SELECT "cake"."name", count("cake"."id") AS "num_of_cakes" FROM "cake""#,
    ///         vec![],
    ///     ))
    ///     .all(&db)
    ///     .await?;
    ///
    /// assert_eq!(
    ///     res,
    ///     vec![
    ///         ("Chocolate Forest".to_owned(), 1),
    ///         ("New York Cheese".to_owned(), 1),
    ///     ]
    /// );
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"SELECT "cake"."name", count("cake"."id") AS "num_of_cakes" FROM "cake""#,
    ///         vec![]
    ///     ),]
    /// );
    /// #
    /// # Ok(())
    /// # }
    /// ```
    fn find_by_statement<C>(stmt: Statement) -> SelectorRaw<SelectGetableValue<Self, C>>
    where
        C: strum::IntoEnumIterator + sea_query::Iden,
    {
        SelectorRaw::<SelectGetableValue<Self, C>>::with_columns(stmt)
    }
}

impl<T> TryGetableMany for T
where
    T: TryGetable,
{
    fn try_get_many(res: &QueryResult, pre: &str, cols: &[String]) -> Result<Self, TryGetError> {
        try_get_many_with_slice_len_of(1, cols)?;
        T::try_get(res, pre, &cols[0])
    }
}

impl<T> TryGetableMany for (T,)
where
    T: TryGetableMany,
{
    fn try_get_many(res: &QueryResult, pre: &str, cols: &[String]) -> Result<Self, TryGetError> {
        T::try_get_many(res, pre, cols).map(|r| (r,))
    }
}

impl<A, B> TryGetableMany for (A, B)
where
    A: TryGetable,
    B: TryGetable,
{
    fn try_get_many(res: &QueryResult, pre: &str, cols: &[String]) -> Result<Self, TryGetError> {
        try_get_many_with_slice_len_of(2, cols)?;
        Ok((
            A::try_get(res, pre, &cols[0])?,
            B::try_get(res, pre, &cols[1])?,
        ))
    }
}

impl<A, B, C> TryGetableMany for (A, B, C)
where
    A: TryGetable,
    B: TryGetable,
    C: TryGetable,
{
    fn try_get_many(res: &QueryResult, pre: &str, cols: &[String]) -> Result<Self, TryGetError> {
        try_get_many_with_slice_len_of(3, cols)?;
        Ok((
            A::try_get(res, pre, &cols[0])?,
            B::try_get(res, pre, &cols[1])?,
            C::try_get(res, pre, &cols[2])?,
        ))
    }
}

impl<A, B, C, D> TryGetableMany for (A, B, C, D)
where
    A: TryGetable,
    B: TryGetable,
    C: TryGetable,
    D: TryGetable,
{
    fn try_get_many(res: &QueryResult, pre: &str, cols: &[String]) -> Result<Self, TryGetError> {
        try_get_many_with_slice_len_of(4, cols)?;
        Ok((
            A::try_get(res, pre, &cols[0])?,
            B::try_get(res, pre, &cols[1])?,
            C::try_get(res, pre, &cols[2])?,
            D::try_get(res, pre, &cols[3])?,
        ))
    }
}

impl<A, B, C, D, E> TryGetableMany for (A, B, C, D, E)
where
    A: TryGetable,
    B: TryGetable,
    C: TryGetable,
    D: TryGetable,
    E: TryGetable,
{
    fn try_get_many(res: &QueryResult, pre: &str, cols: &[String]) -> Result<Self, TryGetError> {
        try_get_many_with_slice_len_of(5, cols)?;
        Ok((
            A::try_get(res, pre, &cols[0])?,
            B::try_get(res, pre, &cols[1])?,
            C::try_get(res, pre, &cols[2])?,
            D::try_get(res, pre, &cols[3])?,
            E::try_get(res, pre, &cols[4])?,
        ))
    }
}

impl<A, B, C, D, E, F> TryGetableMany for (A, B, C, D, E, F)
where
    A: TryGetable,
    B: TryGetable,
    C: TryGetable,
    D: TryGetable,
    E: TryGetable,
    F: TryGetable,
{
    fn try_get_many(res: &QueryResult, pre: &str, cols: &[String]) -> Result<Self, TryGetError> {
        try_get_many_with_slice_len_of(6, cols)?;
        Ok((
            A::try_get(res, pre, &cols[0])?,
            B::try_get(res, pre, &cols[1])?,
            C::try_get(res, pre, &cols[2])?,
            D::try_get(res, pre, &cols[3])?,
            E::try_get(res, pre, &cols[4])?,
            F::try_get(res, pre, &cols[5])?,
        ))
    }
}

fn try_get_many_with_slice_len_of(len: usize, cols: &[String]) -> Result<(), TryGetError> {
    if cols.len() < len {
        Err(TryGetError::DbErr(DbErr::Query(format!(
            "Expect {} column names supplied but got slice of length {}",
            len,
            cols.len()
        ))))
    } else {
        Ok(())
    }
}

// TryGetableFromJson //

/// Perform a query on multiple columns
#[cfg(feature = "with-json")]
pub trait TryGetableFromJson: Sized
where
    for<'de> Self: serde::Deserialize<'de>,
{
    /// Ensure the type implements this method
    fn try_get_from_json(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let column = format!("{}{}", pre, col);
        let res = match &res.row {
            #[cfg(feature = "sqlx-mysql")]
            QueryResultRow::SqlxMySql(row) => {
                use sqlx::Row;
                row.try_get::<Option<serde_json::Value>, _>(column.as_str())
                    .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                    .and_then(|opt| opt.ok_or(TryGetError::Null))
            }
            #[cfg(feature = "sqlx-postgres")]
            QueryResultRow::SqlxPostgres(row) => {
                use sqlx::Row;
                row.try_get::<Option<serde_json::Value>, _>(column.as_str())
                    .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                    .and_then(|opt| opt.ok_or(TryGetError::Null))
            }
            #[cfg(feature = "sqlx-sqlite")]
            QueryResultRow::SqlxSqlite(row) => {
                use sqlx::Row;
                row.try_get::<Option<serde_json::Value>, _>(column.as_str())
                    .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                    .and_then(|opt| opt.ok_or(TryGetError::Null))
            }
            #[cfg(feature = "mock")]
            QueryResultRow::Mock(row) => {
                row.try_get::<serde_json::Value>(column.as_str())
                    .map_err(|e| {
                        debug_print!("{:#?}", e.to_string());
                        TryGetError::Null
                    })
            }
            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        };
        res.and_then(|json| {
            serde_json::from_value(json).map_err(|e| TryGetError::DbErr(DbErr::Json(e.to_string())))
        })
    }
}

#[cfg(feature = "with-json")]
impl<T> TryGetable for T
where
    T: TryGetableFromJson,
{
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        T::try_get_from_json(res, pre, col)
    }
}

// TryFromU64 //
/// Try to convert a type to a u64
pub trait TryFromU64: Sized {
    /// The method to convert the type to a u64
    fn try_from_u64(n: u64) -> Result<Self, DbErr>;
}

macro_rules! try_from_u64_err {
    ( $type: ty ) => {
        impl TryFromU64 for $type {
            fn try_from_u64(_: u64) -> Result<Self, DbErr> {
                Err(DbErr::Exec(format!(
                    "{} cannot be converted from u64",
                    stringify!($type)
                )))
            }
        }
    };

    ( $($gen_type: ident),* ) => {
        impl<$( $gen_type, )*> TryFromU64 for ($( $gen_type, )*)
        where
            $( $gen_type: TryFromU64, )*
        {
            fn try_from_u64(_: u64) -> Result<Self, DbErr> {
                Err(DbErr::Exec(format!(
                    "{} cannot be converted from u64",
                    stringify!(($($gen_type,)*))
                )))
            }
        }
    };
}

// impl TryFromU64 for tuples with generic types
try_from_u64_err!(A, B);
try_from_u64_err!(A, B, C);
try_from_u64_err!(A, B, C, D);
try_from_u64_err!(A, B, C, D, E);
try_from_u64_err!(A, B, C, D, E, F);

macro_rules! try_from_u64_numeric {
    ( $type: ty ) => {
        impl TryFromU64 for $type {
            fn try_from_u64(n: u64) -> Result<Self, DbErr> {
                use std::convert::TryInto;
                n.try_into().map_err(|_| {
                    DbErr::Exec(format!(
                        "fail to convert '{}' into '{}'",
                        n,
                        stringify!($type)
                    ))
                })
            }
        }
    };
}

try_from_u64_numeric!(i8);
try_from_u64_numeric!(i16);
try_from_u64_numeric!(i32);
try_from_u64_numeric!(i64);
try_from_u64_numeric!(u8);
try_from_u64_numeric!(u16);
try_from_u64_numeric!(u32);
try_from_u64_numeric!(u64);

macro_rules! try_from_u64_string {
    ( $type: ty ) => {
        impl TryFromU64 for $type {
            fn try_from_u64(n: u64) -> Result<Self, DbErr> {
                Ok(n.to_string())
            }
        }
    };
}

try_from_u64_string!(String);

try_from_u64_err!(bool);
try_from_u64_err!(f32);
try_from_u64_err!(f64);
try_from_u64_err!(Vec<u8>);

#[cfg(feature = "with-json")]
try_from_u64_err!(serde_json::Value);

#[cfg(feature = "with-chrono")]
try_from_u64_err!(chrono::NaiveDate);

#[cfg(feature = "with-chrono")]
try_from_u64_err!(chrono::NaiveTime);

#[cfg(feature = "with-chrono")]
try_from_u64_err!(chrono::NaiveDateTime);

#[cfg(feature = "with-chrono")]
try_from_u64_err!(chrono::DateTime<chrono::FixedOffset>);

#[cfg(feature = "with-chrono")]
try_from_u64_err!(chrono::DateTime<chrono::Utc>);

#[cfg(feature = "with-chrono")]
try_from_u64_err!(chrono::DateTime<chrono::Local>);

#[cfg(feature = "with-rust_decimal")]
try_from_u64_err!(rust_decimal::Decimal);

#[cfg(feature = "with-uuid")]
try_from_u64_err!(uuid::Uuid);
