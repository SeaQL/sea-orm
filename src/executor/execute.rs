#[cfg(any(feature = "sqlx-mysql", feature = "sqlx-sqlite", feature = "mock"))]
#[derive(Debug)]
pub struct ExecResult {
    pub(crate) result: ExecResultHolder,
}

#[cfg(feature = "sqlx-postgres")]
#[derive(Debug)]
pub struct ExecResult<T> {
    pub(crate) result: ExecResultHolder<T>,
}

#[cfg(feature = "sqlx-mysql")]
#[derive(Debug)]
pub(crate) struct ExecResultHolder(sqlx::mysql::MySqlQueryResult);

#[cfg(feature = "sqlx-postgres")]
#[derive(Debug)]
pub(crate) struct ExecResultHolder<T> {
    pub(crate) last_insert_id: Option<T>,
    pub(crate) rows_affected: u64,
}

#[cfg(feature = "sqlx-sqlite")]
#[derive(Debug)]
pub(crate) struct ExecResultHolder(sqlx::sqlite::SqliteQueryResult);

#[cfg(feature = "mock")]
#[derive(Debug)]
pub(crate) struct ExecResultHolder(pub(crate) crate::MockExecResult);

// ExecResult //

macro_rules! impl_exec_result {
    ( T, |$ident_one:ident| $b_one: block, |$ident_two:ident| $b_two: block ) => {
        impl<T> ExecResult<T>
        where
            T: Clone,
        {
            pub fn last_insert_id(&$ident_one) -> Option<T> {
                $b_one
            }

            pub fn rows_affected(&$ident_two) -> u64 {
                $b_two
            }
        }
    };

    ( $return_type: ty, |$ident_one:ident| $b_one: block, |$ident_two:ident| $b_two: block ) => {
        impl ExecResult {
            pub fn last_insert_id(&$ident_one) -> $return_type {
                $b_one
            }

            pub fn rows_affected(&$ident_two) -> u64 {
                $b_two
            }
        }
    };
}

#[cfg(feature = "sqlx-mysql")]
impl_exec_result!(u64, |self| { self.result.0.last_insert_id() }, |self| {
    self.result.0.rows_affected()
});

#[cfg(feature = "sqlx-postgres")]
impl_exec_result!(T, |self| { self.result.last_insert_id.clone() }, |self| {
    self.result.rows_affected
});

#[cfg(feature = "sqlx-sqlite")]
impl_exec_result!(
    u64,
    |self| {
        let last_insert_rowid = self.result.0.last_insert_rowid();
        if last_insert_rowid < 0 {
            panic!("negative last_insert_rowid")
        } else {
            last_insert_rowid as u64
        }
    },
    |self| { self.result.0.rows_affected() }
);

#[cfg(feature = "mock")]
impl_exec_result!(u64, |self| { self.result.0.last_insert_id }, |self| {
    self.result.0.rows_affected
});
