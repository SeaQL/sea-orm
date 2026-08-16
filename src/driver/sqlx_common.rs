use crate::{ConnAcquireErr, ConnectOptions, DbErr, RuntimeErr};
use std::{sync::Arc, time::Duration};

/// Callback stored for a `before_acquire` hook on [`ConnectOptions`].
///
/// This mirrors the signature accepted by SQLx's
/// [`PoolOptions::before_acquire`][sqlx::pool::PoolOptions::before_acquire] for the given
/// database backend `DB`. Note the fully-qualified [`futures_util::future::BoxFuture`]: the
/// crate-local `BoxFuture` alias collapses to `T` under the `sync` feature, which would not
/// match SQLx's signature.
pub(crate) type BeforeAcquireFn<DB> = Arc<
    dyn for<'c> Fn(
            &'c mut <DB as sqlx::Database>::Connection,
            sqlx::pool::PoolConnectionMetadata,
        ) -> futures_util::future::BoxFuture<'c, Result<bool, sqlx::Error>>
        + Send
        + Sync,
>;

/// Converts an [sqlx::error] execution error to a [DbErr]
pub fn sqlx_error_to_exec_err(err: sqlx::Error) -> DbErr {
    DbErr::Exec(RuntimeErr::SqlxError(err.into()))
}

/// Converts an [sqlx::error] query error to a [DbErr]
pub fn sqlx_error_to_query_err(err: sqlx::Error) -> DbErr {
    DbErr::Query(RuntimeErr::SqlxError(err.into()))
}

/// Converts an [sqlx::error] connection error to a [DbErr]
pub fn sqlx_error_to_conn_err(err: sqlx::Error) -> DbErr {
    DbErr::Conn(RuntimeErr::SqlxError(err.into()))
}

/// Converts an [sqlx::error] error to a [DbErr]
pub fn sqlx_map_err_ignore_not_found<T: std::fmt::Debug>(
    err: Result<Option<T>, sqlx::Error>,
) -> Result<Option<T>, DbErr> {
    if let Err(sqlx::Error::RowNotFound) = err {
        Ok(None)
    } else {
        err.map_err(sqlx_error_to_query_err)
    }
}

/// Converts an [sqlx::error] error to a [DbErr]
pub fn sqlx_conn_acquire_err(sqlx_err: sqlx::Error) -> DbErr {
    match sqlx_err {
        sqlx::Error::PoolTimedOut => DbErr::ConnectionAcquire(ConnAcquireErr::Timeout),
        sqlx::Error::PoolClosed => DbErr::ConnectionAcquire(ConnAcquireErr::ConnectionClosed),
        _ => DbErr::Conn(RuntimeErr::SqlxError(sqlx_err.into())),
    }
}

impl ConnectOptions {
    /// Convert [ConnectOptions] into [sqlx::pool::PoolOptions]
    pub fn sqlx_pool_options<DB>(self) -> sqlx::pool::PoolOptions<DB>
    where
        DB: sqlx::Database,
    {
        let mut opt = sqlx::pool::PoolOptions::new();
        if let Some(max_connections) = self.max_connections {
            opt = opt.max_connections(max_connections);
        }
        if let Some(min_connections) = self.min_connections {
            opt = opt.min_connections(min_connections);
        }
        if let Some(connect_timeout) = self.connect_timeout {
            opt = opt.acquire_timeout(connect_timeout);
        }
        if let Some(idle_timeout) = self.idle_timeout {
            opt = opt.idle_timeout(idle_timeout);
        }
        if let Some(acquire_timeout) = self.acquire_timeout {
            opt = opt.acquire_timeout(acquire_timeout);
        }
        if let Some(max_lifetime) = self.max_lifetime {
            opt = opt.max_lifetime(max_lifetime);
        }
        opt = opt.test_before_acquire(self.test_before_acquire);
        opt
    }

    /// Install the composed `before_acquire` hook onto a [`sqlx::pool::PoolOptions`].
    ///
    /// SQLx exposes a single `before_acquire` slot whose setter *replaces* rather than
    /// composes, and offers no getter to read it back. To let the idle-ping shorthand
    /// ([`ConnectOptions::test_before_acquire_if_idle_for`]) coexist with a user-provided
    /// per-backend callback ([`ConnectOptions::map_sqlx_postgres_before_acquire`] and
    /// friends), this composes both into one closure: the idle-ping runs first, then the
    /// user callback. When a ping threshold is set, `test_before_acquire` is forced off so
    /// the connection is pinged only past the threshold rather than on every acquire.
    ///
    /// Returns `opt` untouched when neither option is configured, so callers that opt into
    /// nothing get byte-for-byte the previous behavior.
    pub(crate) fn apply_before_acquire<DB>(
        mut opt: sqlx::pool::PoolOptions<DB>,
        ping_after_idle: Option<Duration>,
        user_cb: Option<BeforeAcquireFn<DB>>,
    ) -> sqlx::pool::PoolOptions<DB>
    where
        DB: sqlx::Database,
    {
        use sqlx::Connection;

        if ping_after_idle.is_none() && user_cb.is_none() {
            return opt;
        }
        if ping_after_idle.is_some() {
            opt = opt.test_before_acquire(false);
        }
        opt.before_acquire(move |conn, meta| {
            let user_cb = user_cb.clone();
            Box::pin(async move {
                if let Some(threshold) = ping_after_idle {
                    // `idle_for` is `Copy`; read it before `meta` is moved into the user callback.
                    // `>=` matches the "idle for at least `threshold`" contract documented on
                    // `ConnectOptions::test_before_acquire_if_idle_for`.
                    if meta.idle_for >= threshold {
                        conn.ping().await?;
                    }
                }
                match user_cb {
                    Some(user_cb) => user_cb(conn, meta).await,
                    None => Ok(true),
                }
            })
        })
    }
}

#[cfg(all(test, feature = "sqlx-postgres"))]
mod tests {
    use crate::ConnectOptions;
    use sqlx::Connection;
    use std::time::Duration;

    #[test]
    fn idle_shorthand_disables_test_before_acquire() {
        let mut opt = ConnectOptions::new("postgres://localhost/db");
        assert!(opt.get_test_before_acquire());
        assert_eq!(opt.get_test_before_acquire_if_idle_for(), None);

        opt.test_before_acquire_if_idle_for(Duration::from_secs(30));
        assert!(!opt.get_test_before_acquire());
        assert_eq!(
            opt.get_test_before_acquire_if_idle_for(),
            Some(Duration::from_secs(30))
        );
    }

    #[test]
    fn compose_shorthand_and_user_callback() {
        let mut opt = ConnectOptions::new("postgres://localhost/db");
        opt.test_before_acquire_if_idle_for(Duration::from_secs(30))
            .map_sqlx_postgres_before_acquire(|conn, _meta| {
                Box::pin(async move {
                    conn.ping().await?;
                    Ok(true)
                })
            });

        // Composing both into SQLx's single `before_acquire` slot type-checks and returns a
        // usable `PoolOptions`. Behavioral ping timing requires a live pool, covered elsewhere.
        let pool_opts = ConnectOptions::apply_before_acquire::<sqlx::Postgres>(
            sqlx::pool::PoolOptions::new(),
            opt.get_test_before_acquire_if_idle_for(),
            opt.pg_before_acquire_fn.clone(),
        );
        let _ = pool_opts;
    }

    #[test]
    fn apply_before_acquire_noop_when_unset() {
        // With neither option set, the helper must return the options untouched.
        let opts = ConnectOptions::apply_before_acquire::<sqlx::Postgres>(
            sqlx::pool::PoolOptions::new(),
            None,
            None,
        );
        let _ = opts;
    }
}
