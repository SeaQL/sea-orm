//! Support for the built-in PostgreSQL geometric types (issue #282):
//! `point`, `line`, `lseg`, `box`, `path`, `polygon`, `circle`.
//!
//! These are the *core Postgres* geometric types (Postgres manual §8.8) — **not**
//! PostGIS. No extension is required. The underlying representation is sqlx's own
//! [`PgPoint`], [`PgLine`], [`PgLSeg`], [`PgBox`], [`PgPath`], [`PgPolygon`] and
//! [`PgCircle`], wrapped in a thin [`Geo<T>`] newtype so the required sea-orm /
//! sea-query traits can be implemented (Rust's orphan rules forbid implementing
//! them directly on the foreign sqlx types).
//!
//! `Geo<T>` derefs to the inner sqlx type, so all its fields/methods are
//! available directly.
//!
//! ## How values cross the DB boundary
//!
//! sea-query's [`Value`] has no geometric variant, so values are bound as their
//! canonical Postgres **text** form and cast on both sides:
//!
//! ```ignore
//! use sea_orm::entity::prelude::*;
//!
//! #[sea_orm(
//!     column_type = r#"custom("point")"#,
//!     select_as = "text",   // CAST(col AS text) -> "(x,y)"
//!     save_as = "point"     // CAST($1 AS point)
//! )]
//! pub location: Geo<PgPoint>,
//! ```

use std::{
    ops::{Deref, DerefMut},
    str::FromStr,
};

use sea_query::{ArrayType, Nullable, ValueType, ValueTypeErr};
use sqlx::postgres::types::{PgBox, PgCircle, PgLSeg, PgLine, PgPath, PgPoint, PgPolygon};

use crate::{self as sea_orm, ColumnType, DbErr, TryFromU64, TryGetError, TryGetable, Value};

/// A PostgreSQL geometric value wrapping one of sqlx's `Pg*` geometric types.
///
/// See the module docs for usage.
#[derive(Clone, Debug, PartialEq)]
pub struct Geo<T: PgGeometry>(pub T);

impl<T: PgGeometry> Geo<T> {
    /// Wrap an sqlx geometric value.
    pub fn new(value: T) -> Self {
        Geo(value)
    }

    /// Unwrap into the inner sqlx geometric value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

/// Trait implemented for each supported sqlx geometric type, describing how it
/// converts to/from its canonical PostgreSQL text form.
pub trait PgGeometry: Sized + Clone {
    /// The PostgreSQL type name (used for the `Custom(..)` column type).
    const PG_TYPE: &'static str;

    /// Render to the canonical PostgreSQL text input form.
    fn to_pg_text(&self) -> String;

    /// Parse from PostgreSQL text output.
    fn from_pg_text(s: &str) -> Result<Self, DbErr>;
}

/// Format a coordinate pair as `(x,y)`.
fn pt(x: f64, y: f64) -> String {
    format!("({x},{y})")
}

fn join_points(points: &[PgPoint]) -> String {
    points
        .iter()
        .map(|p| pt(p.x, p.y))
        .collect::<Vec<_>>()
        .join(",")
}

macro_rules! impl_pg_geometry {
    ($ty:ty, $name:literal, $to_text:expr) => {
        impl PgGeometry for $ty {
            const PG_TYPE: &'static str = $name;

            fn to_pg_text(&self) -> String {
                let f: &dyn Fn(&$ty) -> String = &$to_text;
                f(self)
            }

            fn from_pg_text(s: &str) -> Result<Self, DbErr> {
                <$ty>::from_str(s).map_err(|e| {
                    DbErr::Type(format!(concat!("Failed to parse ", $name, ": {}"), e))
                })
            }
        }
    };
}

impl_pg_geometry!(PgPoint, "point", |p| pt(p.x, p.y));
impl_pg_geometry!(PgLine, "line", |l| format!("{{{},{},{}}}", l.a, l.b, l.c));
impl_pg_geometry!(PgLSeg, "lseg", |l| format!(
    "[{},{}]",
    pt(l.start_x, l.start_y),
    pt(l.end_x, l.end_y)
));
impl_pg_geometry!(PgBox, "box", |b| format!(
    "({},{})",
    pt(b.upper_right_x, b.upper_right_y),
    pt(b.lower_left_x, b.lower_left_y)
));
impl_pg_geometry!(PgCircle, "circle", |c| format!(
    "<{},{}>",
    pt(c.x, c.y),
    c.radius
));
impl_pg_geometry!(PgPath, "path", |p: &PgPath| {
    let inner = join_points(&p.points);
    if p.closed {
        format!("({inner})")
    } else {
        format!("[{inner}]")
    }
});
impl_pg_geometry!(PgPolygon, "polygon", |p: &PgPolygon| format!(
    "({})",
    join_points(&p.points)
));

// ---- sea-orm / sea-query trait impls for the local `Geo<T>` newtype ----

impl<T: PgGeometry> From<Geo<T>> for Value {
    fn from(value: Geo<T>) -> Self {
        Value::String(Some(value.0.to_pg_text()))
    }
}

impl<T: PgGeometry> TryGetable for Geo<T> {
    fn try_get_by<I: sea_orm::ColIdx>(
        res: &sea_orm::QueryResult,
        index: I,
    ) -> Result<Self, TryGetError> {
        // Column is selected with `select_as = "text"`, so we read the canonical
        // text form and parse it with sqlx's own `FromStr`. Read as `Option` so a
        // SQL NULL surfaces as `TryGetError::Null` (catchable by `Option<Geo<T>>`).
        let text: Option<String> = res.try_get_by(index)?;
        match text {
            Some(text) => T::from_pg_text(&text).map(Geo).map_err(TryGetError::DbErr),
            None => Err(TryGetError::Null(format!("{index:?}"))),
        }
    }
}

impl<T: PgGeometry> ValueType for Geo<T> {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        match v {
            Value::String(Some(s)) => T::from_pg_text(&s).map(Geo).map_err(|_| ValueTypeErr),
            _ => Err(ValueTypeErr),
        }
    }

    fn type_name() -> String {
        format!("Geo<{}>", T::PG_TYPE)
    }

    fn array_type() -> ArrayType {
        ArrayType::String
    }

    fn column_type() -> ColumnType {
        ColumnType::custom(T::PG_TYPE)
    }
}

impl<T: PgGeometry> Nullable for Geo<T> {
    fn null() -> Value {
        Value::String(None)
    }
}

impl<T: PgGeometry> TryFromU64 for Geo<T> {
    fn try_from_u64(_n: u64) -> Result<Self, DbErr> {
        Err(DbErr::ConvertFromU64("Geo"))
    }
}

impl<T: PgGeometry> sea_orm::IntoActiveValue<Geo<T>> for Geo<T> {
    fn into_active_value(self) -> crate::ActiveValue<Geo<T>> {
        crate::ActiveValue::Set(self)
    }
}

impl<T: PgGeometry> From<T> for Geo<T> {
    fn from(value: T) -> Self {
        Geo(value)
    }
}

impl<T: PgGeometry> Deref for Geo<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: PgGeometry> DerefMut for Geo<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}
