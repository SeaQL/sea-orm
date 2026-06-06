//! Positive tests for `PkAutoIncrementHint` resolution.
//!
//! These pin the contract that `DeriveEntityModel` emits the trait call
//! correctly and that the trait propagates through `DeriveValueType`
//! wrappers and `Id<E, T>` aliases.

use sea_orm::{DeriveValueType, Id, PkAutoIncrementHint, PrimaryKeyTrait, entity::prelude::*};

#[derive(Clone, Debug, PartialEq, Eq, DeriveValueType)]
pub struct IntegerWrapper(pub i64);

#[derive(Clone, Debug, PartialEq, Eq, DeriveValueType)]
pub struct StringWrapper(pub String);

#[derive(Clone, Debug, PartialEq, Eq, DeriveValueType)]
pub struct NestedIntegerWrapper(pub IntegerWrapper);

#[derive(Clone, Debug, PartialEq, Eq, DeriveValueType)]
pub struct NestedStringWrapper(pub StringWrapper);

mod ent_for_id {
    use super::*;
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "ent_for_id")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}

/// Entity whose PK is an `Id<E, T>` alias (the shape
/// `sea-orm-cli generate --with-pk-newtypes` produces). Exercises
/// trait resolution one layer up from raw scalars: the macro emits
/// `<EntId as PkAutoIncrementHint>::IS_AUTO`, which resolves via the
/// `DelegatesPkAutoIncrementHint` blanket on `Id<E, T>` down to the
/// inner `i64`.
mod ent_for_typed_pk {
    use sea_orm::entity::prelude::*;
    pub type EntId = sea_orm::Id<Entity, i64>;
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "ent_for_typed_pk")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: EntId,
        pub name: String,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}

/// Entity with a composite PK whose components are themselves typed
/// `Id<E, T>` aliases. The macro must short-circuit to `false`
/// regardless of how the trait would resolve for the individual
/// component types.
mod ent_with_composite_pk {
    use sea_orm::entity::prelude::*;
    pub type LeftId = sea_orm::Id<Entity, i64>;
    pub type RightId = sea_orm::Id<Entity, i64>;
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "ent_with_composite_pk")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub left_id: LeftId,
        #[sea_orm(primary_key)]
        pub right_id: RightId,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}

/// Entity whose PK is a bare `type X = i32` alias (not an `Id<E, T>`).
/// A transparent alias resolves identically to its target, so the macro
/// emits `<BareUserId as PkAutoIncrementHint>::IS_AUTO`, which is the
/// `i32` impl and yields `true`.
mod ent_for_bare_alias {
    use super::*;
    pub type BareUserId = i32;
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "ent_for_bare_alias")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: BareUserId,
        pub name: String,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}

#[test]
fn primitive_integer_defaults_true() {
    assert!(<i32 as PkAutoIncrementHint>::IS_AUTO);
    assert!(<i64 as PkAutoIncrementHint>::IS_AUTO);
}

#[test]
fn primitive_string_defaults_false() {
    assert!(!<String as PkAutoIncrementHint>::IS_AUTO);
}

#[test]
fn value_type_wrapper_propagates_integer() {
    assert!(<IntegerWrapper as PkAutoIncrementHint>::IS_AUTO);
}

#[test]
fn value_type_wrapper_propagates_string() {
    assert!(!<StringWrapper as PkAutoIncrementHint>::IS_AUTO);
}

#[test]
fn value_type_wrapper_propagates_through_nested() {
    assert!(<NestedIntegerWrapper as PkAutoIncrementHint>::IS_AUTO);
    assert!(!<NestedStringWrapper as PkAutoIncrementHint>::IS_AUTO);
}

#[test]
fn id_alias_propagates_through_inner() {
    type IntId = Id<ent_for_id::Entity, i32>;
    type StrId = Id<ent_for_id::Entity, String>;
    assert!(<IntId as PkAutoIncrementHint>::IS_AUTO);
    assert!(!<StrId as PkAutoIncrementHint>::IS_AUTO);
}

#[test]
fn entity_with_i32_pk_resolves_true() {
    assert!(<ent_for_id::PrimaryKey as PrimaryKeyTrait>::auto_increment());
}

#[test]
fn entity_with_id_alias_pk_resolves_true() {
    assert!(<ent_for_typed_pk::PrimaryKey as PrimaryKeyTrait>::auto_increment());
}

#[test]
fn entity_with_bare_alias_i32_pk_resolves_true() {
    assert!(<ent_for_bare_alias::PrimaryKey as PrimaryKeyTrait>::auto_increment());
}

#[test]
fn composite_pk_is_never_auto_increment() {
    assert!(!<ent_with_composite_pk::PrimaryKey as PrimaryKeyTrait>::auto_increment());
}

#[cfg(feature = "with-uuid")]
#[test]
fn uuid_defaults_false() {
    assert!(!<uuid::Uuid as PkAutoIncrementHint>::IS_AUTO);
}
