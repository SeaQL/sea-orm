use std::marker::PhantomData;
use std::ops::{DerefMut};

use rocket::{error, info_, Build, Ignite, Phase, Rocket, Sentinel};
use rocket::fairing::{self, Fairing, Info, Kind};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::http::Status;

use rocket::yansi::Paint;
use rocket::figment::providers::Serialized;

use crate::Pool;

/// Derivable trait which ties a database [`Pool`] with a configuration name.
///
/// This trait should rarely, if ever, be implemented manually. Instead, it
/// should be derived:
///
/// ```ignore
/// use sea_orm_rocket::{Database};
/// # use sea_orm_rocket::MockPool as SeaOrmPool;
///
/// #[derive(Database, Debug)]
/// #[database("sea_orm")]
/// struct Db(SeaOrmPool);
///
/// #[launch]
/// fn rocket() -> _ {
///     rocket::build().attach(Db::init())
/// }
/// ```
///
/// See the [`Database` derive](derive@crate::Database) for details.
pub trait Database: From<Self::Pool> + DerefMut<Target = Self::Pool> + Send + Sync + 'static {
    /// The [`Pool`] type of connections to this database.
    ///
    /// When `Database` is derived, this takes the value of the `Inner` type in
    /// `struct Db(Inner)`.
    type Pool: Pool;

    /// The configuration name for this database.
    ///
    /// When `Database` is derived, this takes the value `"name"` in the
    /// `#[database("name")]` attribute.
    const NAME: &'static str;

    /// Returns a fairing that initializes the database and its connection pool.
    ///
    /// # Example
    ///
    /// ```rust
    /// # mod _inner {
    /// # use rocket::launch;
    /// use sea_orm_rocket::{Database};
    /// # use sea_orm_rocket::MockPool as SeaOrmPool;
    ///
    /// #[derive(Database)]
    /// #[database("sea_orm")]
    /// struct Db(SeaOrmPool);
    ///
    /// #[launch]
    /// fn rocket() -> _ {
    ///     rocket::build().attach(Db::init())
    /// }
    /// # }
    /// ```
    fn init() -> Initializer<Self> {
        Initializer::new()
    }

    /// Returns a reference to the initialized database in `rocket`. The
    /// initializer fairing returned by `init()` must have already executed for
    /// `Option` to be `Some`. This is guaranteed to be the case if the fairing
    /// is attached and either:
    ///
    ///   * Rocket is in the [`Orbit`](rocket::Orbit) phase. That is, the
    ///     application is running. This is always the case in request guards
    ///     and liftoff fairings,
    ///   * _or_ Rocket is in the [`Build`](rocket::Build) or
    ///     [`Ignite`](rocket::Ignite) phase and the `Initializer` fairing has
    ///     already been run. This is the case in all fairing callbacks
    ///     corresponding to fairings attached _after_ the `Initializer`
    ///     fairing.
    ///
    /// # Example
    ///
    /// Run database migrations in an ignite fairing. It is imperative that the
    /// migration fairing be registered _after_ the `init()` fairing.
    ///
    /// ```rust
    /// # mod _inner {
    /// # use rocket::launch;
    /// use rocket::{Rocket, Build};
    /// use rocket::fairing::{self, AdHoc};
    ///
    /// use sea_orm_rocket::{Database};
    /// # use sea_orm_rocket::MockPool as SeaOrmPool;
    ///
    /// #[derive(Database)]
    /// #[database("sea_orm")]
    /// struct Db(SeaOrmPool);
    ///
    /// async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    ///     if let Some(db) = Db::fetch(&rocket) {
    ///         // run migrations using `db`. get the inner type with &db.0.
    ///         Ok(rocket)
    ///     } else {
    ///         Err(rocket)
    ///     }
    /// }
    ///
    /// #[launch]
    /// fn rocket() -> _ {
    ///     rocket::build()
    ///         .attach(Db::init())
    ///         .attach(AdHoc::try_on_ignite("DB Migrations", run_migrations))
    /// }
    /// # }
    /// ```
    fn fetch<P: Phase>(rocket: &Rocket<P>) -> Option<&Self> {
        if let Some(db) = rocket.state() {
            return Some(db);
        }

        let dbtype = std::any::type_name::<Self>();
        let fairing = Paint::default(format!("{}::init()", dbtype)).bold();
        error!("Attempted to fetch unattached database `{}`.", Paint::default(dbtype).bold());
        info_!("`{}` fairing must be attached prior to using this database.", fairing);
        None
    }
}

/// A [`Fairing`] which initializes a [`Database`] and its connection pool.
///
/// A value of this type can be created for any type `D` that implements
/// [`Database`] via the [`Database::init()`] method on the type. Normally, a
/// value of this type _never_ needs to be constructed directly. This
/// documentation exists purely as a reference.
///
/// This fairing initializes a database pool. Specifically, it:
///
///   1. Reads the configuration at `database.db_name`, where `db_name` is
///      [`Database::NAME`].
///
///   2. Sets [`Config`](crate::Config) defaults on the configuration figment.
///
///   3. Calls [`Pool::init()`].
///
///   4. Stores the database instance in managed storage, retrievable via
///      [`Database::fetch()`].
///
/// The name of the fairing itself is `Initializer<D>`, with `D` replaced with
/// the type name `D` unless a name is explicitly provided via
/// [`Self::with_name()`].
pub struct Initializer<D: Database>(Option<&'static str>, PhantomData<fn() -> D>);

/// A request guard which retrieves a single connection to a [`Database`].
///
/// For a database type of `Db`, a request guard of `Connection<Db>` retrieves a
/// single connection to `Db`.
///
/// The request guard succeeds if the database was initialized by the
/// [`Initializer`] fairing and a connection is available within
/// [`connect_timeout`](crate::Config::connect_timeout) seconds.
///   * If the `Initializer` fairing was _not_ attached, the guard _fails_ with
///   status `InternalServerError`. A [`Sentinel`] guards this condition, and so
///   this type of failure is unlikely to occur. A `None` error is returned.
///   * If a connection is not available within `connect_timeout` seconds or
///   another error occurs, the gaurd _fails_ with status `ServiceUnavailable`
///   and the error is returned in `Some`.
///
pub struct Connection<'a, D: Database>(&'a <D::Pool as Pool>::Connection);

impl<D: Database> Initializer<D> {
    /// Returns a database initializer fairing for `D`.
    ///
    /// This method should never need to be called manually. See the [crate
    /// docs](crate) for usage information.
    pub fn new() -> Self {
        Self(None, std::marker::PhantomData)
    }

    /// Returns a database initializer fairing for `D` with name `name`.
    ///
    /// This method should never need to be called manually. See the [crate
    /// docs](crate) for usage information.
    pub fn with_name(name: &'static str) -> Self {
        Self(Some(name), std::marker::PhantomData)
    }
}

impl<'a, D: Database> Connection<'a, D> {
    /// Returns the internal connection value. See the [`Connection` Deref
    /// column](crate#supported-drivers) for the expected type of this value.
    pub fn into_inner(self) -> &'a <D::Pool as Pool>::Connection {
        self.0
    }
}

#[rocket::async_trait]
impl<D: Database> Fairing for Initializer<D> {
    fn info(&self) -> Info {
        Info {
            name: self.0.unwrap_or_else(std::any::type_name::<Self>),
            kind: Kind::Ignite,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        let workers: usize = rocket.figment()
            .extract_inner(rocket::Config::WORKERS)
            .unwrap_or_else(|_| rocket::Config::default().workers);

        let figment = rocket.figment()
            .focus(&format!("databases.{}", D::NAME))
            .merge(Serialized::default("max_connections", workers * 4))
            .merge(Serialized::default("connect_timeout", 5));

        match <D::Pool>::init(&figment).await {
            Ok(pool) => Ok(rocket.manage(D::from(pool))),
            Err(e) => {
                error!("failed to initialize database: {}", e);
                Err(rocket)
            }
        }
    }
}

#[rocket::async_trait]
impl<'r, D: Database> FromRequest<'r> for Connection<'r, D> {
    type Error = Option<<D::Pool as Pool>::Error>;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match D::fetch(req.rocket()) {
            Some(pool) => Outcome::Success(Connection(pool.borrow())),
            None => Outcome::Failure((Status::InternalServerError, None)),
        }
    }
}

impl<D: Database> Sentinel for Connection<'_, D> {
    fn abort(rocket: &Rocket<Ignite>) -> bool {
        D::fetch(rocket).is_none()
    }
}
