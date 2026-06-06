//! Phantom-typed primary-key handle.
//!
//! `Id<E, T>` wraps a primary-key value of underlying type `T` and tags it
//! with the entity `E` at the type level. Two `Id` types with different
//! entity tags are never inter-convertible, so the compiler rejects
//! cross-entity ID confusion at use sites, e.g. passing a
//! `Id<post::Entity, _>` to `user::Entity::find_by_id` is a type error.
//!
//! `Id` is reachable as `sea_orm::Id` and is intentionally absent from the
//! entity prelude: hand-written aliases spell it `sea_orm::Id<Entity, T>`.
//!
//! ## Type parameters
//!
//! `T` is always the raw scalar, `Id<E, T>::value: T`. Keeping the scalar
//! as an explicit type parameter (rather than projecting it through an
//! associated type on `E`) keeps `PrimaryKey::ValueType = Id<E, T>` from
//! becoming infinitely recursive (`Id<E>::Inner = Id<E>::Inner = …`),
//! since the alias spells `T` directly:
//! `pub type CakePk = sea_orm::Id<Entity, i32>;`.
//!
//! ## Usage
//!
//! ```ignore
//! use sea_orm::entity::prelude::*;
//!
//! // Codegen emits this as a one-line alias per entity:
//! pub type CakePk = sea_orm::Id<Entity, i32>;
//!
//! // The model field uses the alias:
//! pub struct Model {
//!     pub id: CakePk,
//!     pub name: String,
//! }
//!
//! // Construction is explicit, `Id::new` (no `From<i32>` blanket):
//! let id = CakePk::new(7);
//!
//! // Queries use the typed handle:
//! let cake = cake::Entity::find_by_id(id).one(db)?;
//! ```
//!
//! ## Safety contract
//!
//! `Id<E, T>` deliberately does NOT impl `From<T>` for any specific scalar.
//! The only construction path is [`Id::new`]. This is what makes
//! `user::Entity::find_by_id(7_i32)` fail to compile when the entity's PK
//! is `Id<user::Entity, i32>`: there's no `i32: Into<Id<user::Entity, i32>>`
//! impl. Do not add such a `From` impl; it re-opens the cross-entity hole
//! this type exists to close.

use crate::{
    ColIdx, DbErr, EntityTrait, PrimaryKeyTrait, QueryResult, TryFromU64, TryGetError, TryGetable,
};
use sea_query::{ArrayType, ColumnType, Nullable, Value, ValueType, ValueTypeErr};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

/// Phantom-typed wrapper around a primary-key value.
///
/// `E` is the entity this id belongs to (a marker, never stored at
/// runtime). `T` is the raw stored value:
/// - For unary PKs, the scalar type (`i32`, `Uuid`, `String`, …).
/// - For composite PKs, a tuple of the typed components
///   (e.g. `(super::cake::CakePk, super::filling::FillingPk)`).
///
/// See the [module-level docs](self) for usage and the safety contract.
#[repr(transparent)]
pub struct Id<E: EntityTrait, T> {
    /// The raw stored value. Public for ergonomic read/unwrap; the
    /// no-`From<T>` contract blocks implicit call-site conversion, not field
    /// access (the entity tag lives in `_marker`, not here, so reading or
    /// mutating `value` cannot turn an `Id<A, _>` into an `Id<B, _>`).
    pub value: T,
    // `PhantomData<fn(E) -> E>` makes `E` invariant (E appears in both
    // parameter and return position), so the compiler never widens an
    // `Id<A, _>` to an `Id<B, _>`. Function-pointer types are always Send +
    // Sync, so this marker preserves those auto-traits on `Id<E, T>`.
    _marker: PhantomData<fn(E) -> E>,
}

impl<E: EntityTrait, T> Id<E, T> {
    /// Wrap a raw value as a typed entity ID. This is the only construction
    /// path, there is no `From<T>` blanket impl, which is what gives
    /// `Id<E, T>` its type-safety contract.
    pub const fn new(value: T) -> Self {
        Self {
            value,
            _marker: PhantomData,
        }
    }

    /// Unwrap to the raw stored value, consuming the wrapper.
    pub fn into_inner(self) -> T {
        self.value
    }
}

// Manual impls so the trait bounds land on `T` (the stored value) rather
// than `E` (a phantom).

impl<E: EntityTrait, T: Clone> Clone for Id<E, T> {
    fn clone(&self) -> Self {
        Self::new(self.value.clone())
    }
}

impl<E: EntityTrait, T: Copy> Copy for Id<E, T> {}

impl<E: EntityTrait, T: fmt::Debug> fmt::Debug for Id<E, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Include the entity tag so `Id<post::Entity, _>(7)` and
        // `Id<user::Entity, _>(7)` don't look identical in logs, that
        // defeats the entire reason this wrapper exists.
        //
        // Every entity struct is named `Entity` by convention, so the
        // disambiguating part is the module that contains it. We render
        // `<parent_module>::<EntityName>`, the last two `::`-segments
        // of `std::any::type_name::<E>()`. Full paths are too verbose
        // for log lines; the trailing two segments preserve the
        // disambiguation while staying readable.
        let full = std::any::type_name::<E>();
        let mut tail = full.rsplitn(3, "::");
        let last = tail.next().unwrap_or(full);
        let prev = tail.next();
        let label = match prev {
            Some(p) => format!("{p}::{last}"),
            None => last.to_owned(),
        };
        write!(f, "Id<{label}>({:?})", self.value)
    }
}

impl<E: EntityTrait, T: PartialEq> PartialEq for Id<E, T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<E: EntityTrait, T: Eq> Eq for Id<E, T> {}

impl<E: EntityTrait, T: Hash> Hash for Id<E, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<E: EntityTrait, T: fmt::Display> fmt::Display for Id<E, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

// === PrimaryKeyTrait::ValueType bounds ======================================
//
// All five trait bounds below delegate to `T`. `Into<Value>` (unary `T`)
// bridges to sea-query's blanket `From<V> for ValueTuple`, auto-deriving
// `Id<E, T>: IntoValueTuple`. Composite PKs never use `Id<E, tuple>`: each
// component is a unary `Id<parent, scalar>` and the tuple is just
// `(CakePk, FillingPk)`.

impl<E: EntityTrait, T> From<Id<E, T>> for Value
where
    T: Into<Value>,
{
    fn from(id: Id<E, T>) -> Self {
        id.value.into()
    }
}

// `FromValueTuple` comes from sea-query's blanket impl once `Into<Value>`
// (above) and `ValueType` (below) are present; for composite `T` neither
// fires, which is intentional.

// `TryGetable` (single-column read) also auto-derives `TryGetableMany` for
// `Id<E, T>` and, via the per-arity macro, for tuples of `Id<E, T>` (what
// composite PKs need). `Id<E, T>` does not impl `TryGetable` for tuple `T`:
// composite PKs use tuples of unary `Id<E, scalar>`, not `Id<E, tuple>`.
impl<E: EntityTrait, T: TryGetable> TryGetable for Id<E, T> {
    fn try_get_by<I: ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
        T::try_get_by(res, idx).map(Id::new)
    }
}

impl<E: EntityTrait, T: TryFromU64> TryFromU64 for Id<E, T> {
    fn try_from_u64(n: u64) -> Result<Self, DbErr> {
        T::try_from_u64(n).map(Id::new)
    }
}

// `PrimaryKeyArity` is auto-derived via the blanket
// `impl<V: TryGetable> PrimaryKeyArity for V { const ARITY = 1 }`.

// `sea_query::ValueType` lets `DeriveEntityModel` call
// `<CakePk as ValueType>::column_type()` for the SQL column type. Only when
// `T: ValueType` (a single scalar); composite PKs ask each column instead.
impl<E: EntityTrait, T: ValueType> ValueType for Id<E, T> {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        T::try_from(v).map(Id::new)
    }

    fn type_name() -> String {
        T::type_name()
    }

    fn array_type() -> ArrayType {
        T::array_type()
    }

    fn column_type() -> ColumnType {
        T::column_type()
    }
}

// `Nullable` so the macro can wrap the column in `Option<Id<E, T>>` for
// nullable FK columns.
impl<E: EntityTrait, T: Nullable> Nullable for Id<E, T> {
    fn null() -> Value {
        T::null()
    }
}

// === FindByIdArg ============================================================
//
// `find_by_id` / `filter_by_id` accept anything convertible to the entity's
// primary-key value type. We could bound that directly with `Into`, but doing
// so makes the compiler's "this argument is wrong" diagnostic
// incomprehensible: it reads something like
//   `the trait bound `Id<user::Entity, i32>: From<Id<post::Entity, i32>>`
//    is not satisfied`,
// burying the two entity types inside generic args of `Into`.
//
// `FindByIdArg<E>` is a thin sea-orm-owned wrapper around that same `Into`
// bound. It exists solely so we can attach `#[diagnostic::on_unimplemented]`
// to it, diagnostics can't be attached to `Into` (a std trait). The blanket
// impl forwards through `Into`, so every existing call site still works
// without change. When the bound *fails*, the user sees a message that names
// the entity and the argument type directly.
//
// MSRV is 1.85; `#[diagnostic::on_unimplemented]` is stable since 1.78.

/// Helper bound used by `find_by_id` / `filter_by_id`.
///
/// Implemented for every `T` that converts into `E`'s primary-key value type
/// via `Into`. This trait exists to provide a better compiler error than the
/// raw `Into` bound when the argument doesn't match, see the module docs.
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be used as a primary-key argument for `{E}`",
    label = "expected `{E}`'s `PrimaryKey::ValueType` (or something convertible to it), got `{Self}`",
    note = "type-safe `Id<E, _>` wrappers deliberately do not impl `From<inner>` to prevent cross-entity ID confusion. Construct ids explicitly with `Id::new(..)` (or the per-entity alias's `::new`), and pass an id belonging to this entity."
)]
pub trait FindByIdArg<E: EntityTrait>: Sized {
    /// Convert this argument into the entity's primary-key value tuple.
    fn into_pk_value(self) -> <E::PrimaryKey as PrimaryKeyTrait>::ValueType;
}

// `do_not_recommend` (stable 1.85) tells rustc not to surface this blanket impl
// in error messages when its where-clause fails. Without it, the user sees a
// confusing message about `From<Id<post::Entity, _>>` not being implemented
// for `Id<user::Entity, _>`, the deeper sub-bound, instead of the
// `on_unimplemented` message on `FindByIdArg` itself.
#[diagnostic::do_not_recommend]
impl<E: EntityTrait, T> FindByIdArg<E> for T
where
    T: Into<<E::PrimaryKey as PrimaryKeyTrait>::ValueType>,
{
    fn into_pk_value(self) -> <E::PrimaryKey as PrimaryKeyTrait>::ValueType {
        self.into()
    }
}

// === Serde ==================================================================
//
// Transparent: `Id<E, T>` serializes as just the inner `T`, not as a
// wrapper object. Gated behind `with-json` like the rest of sea-orm's serde
// surface (see `entity/compound/has_one.rs` for the same pattern).

#[cfg(feature = "with-json")]
impl<E: EntityTrait, T: serde::Serialize> serde::Serialize for Id<E, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.value.serialize(serializer)
    }
}

#[cfg(feature = "with-json")]
impl<'de, E: EntityTrait, T: serde::Deserialize<'de>> serde::Deserialize<'de> for Id<E, T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        T::deserialize(deserializer).map(Id::new)
    }
}
