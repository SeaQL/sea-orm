//! Runtime behaviour tests for the `Id<E, T>` primary-key newtype.
//!
//! These pin the behaviour the type-safety contract relies on:
//!   - Layout: `#[repr(transparent)]` holds, so `Id<E, T>` and
//!     `Option<Id<E, T>>` are the same size as the raw scalar.
//!   - Hashability: typed PKs of different entities coexist in distinct
//!     `HashMap`s keyed by their newtype.
//!   - Display/Debug: `Display` delegates to the inner value; `Debug`
//!     includes the entity tag so different entities render distinctly.
//!   - `into_inner` round-trips back to the raw scalar.
//!   - Serde shape: `Id<E, T>` is transparent (a bare number/string, not an
//!     object), including `Option<Id<E, T>>` and a `String`-typed alias.
//!
//! Cross-entity confusion (e.g. comparing or passing the wrong entity's id)
//! is a compile error, pinned separately by the trybuild fixtures under
//! `tests/value_type_pk_compile_fail/`.
//!
//! Two minimal local entities (`post`, `user`) keep this file self-contained.

use std::collections::HashMap;
use std::mem::size_of;

mod post {
    use sea_orm::entity::prelude::*;

    pub type PostId = sea_orm::Id<Entity, i32>;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "post")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: PostId,
        pub title: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

mod user {
    use sea_orm::entity::prelude::*;

    pub type UserId = sea_orm::Id<Entity, i32>;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "user")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: UserId,
        pub name: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

#[test]
fn layout_is_repr_transparent() {
    assert_eq!(size_of::<post::PostId>(), size_of::<i32>());
    assert_eq!(size_of::<user::UserId>(), size_of::<i32>());
    assert_eq!(size_of::<Option<post::PostId>>(), size_of::<Option<i32>>());
}

#[test]
fn typed_ids_are_hashable_in_separate_maps() {
    let mut posts: HashMap<post::PostId, &str> = HashMap::new();
    let mut users: HashMap<user::UserId, &str> = HashMap::new();
    posts.insert(post::PostId::new(7), "post-7");
    users.insert(user::UserId::new(7), "user-7");
    assert_eq!(posts.get(&post::PostId::new(7)), Some(&"post-7"));
    assert_eq!(users.get(&user::UserId::new(7)), Some(&"user-7"));
    // Same inner value, different keyspace.
    assert_eq!(posts.len(), 1);
    assert_eq!(users.len(), 1);
}

#[test]
fn copy_and_clone_only_when_inner_supports_them() {
    // i32 is Copy → PostId is Copy.
    let a = post::PostId::new(3);
    let b = a;
    assert_eq!(a, b);
    let c = a.clone();
    assert_eq!(a, c);
}

#[test]
fn into_inner_round_trip() {
    let id = post::PostId::new(42);
    assert_eq!(id.into_inner(), 42i32);
}

#[test]
fn display_delegates_to_inner() {
    let id = post::PostId::new(101);
    assert_eq!(format!("{}", id), "101");
}

#[test]
fn debug_includes_entity_tag() {
    // The Debug impl prints the entity tag so that
    // `Id<post::Entity, _>(7)` and `Id<user::Entity, _>(7)` don't render
    // identically in logs. The tag is the last two `::` segments of
    // `type_name::<E>()` to disambiguate entities that all conventionally
    // name their inner struct `Entity`.
    let post_id = post::PostId::new(7);
    let user_id = user::UserId::new(7);
    let post_dbg = format!("{post_id:?}");
    let user_dbg = format!("{user_id:?}");
    assert!(post_dbg.contains("post::Entity"), "got: {post_dbg}");
    assert!(user_dbg.contains("user::Entity"), "got: {user_dbg}");
    assert_ne!(
        post_dbg, user_dbg,
        "Debug output must distinguish post::Entity from user::Entity"
    );
    assert!(post_dbg.ends_with("(7)"));
}

#[cfg(feature = "with-json")]
mod serde_shape {
    use super::*;

    #[test]
    fn typed_id_serialises_as_bare_number() {
        let id = post::PostId::new(7);
        let v = serde_json::to_value(&id).expect("serialise");
        assert_eq!(v, serde_json::json!(7));
    }

    #[test]
    fn typed_id_deserialises_from_bare_number() {
        let v = serde_json::json!(13);
        let id: post::PostId = serde_json::from_value(v).expect("deserialise");
        assert_eq!(id, post::PostId::new(13));
    }

    #[test]
    fn option_typed_id_serialises_null_and_some() {
        let none: Option<post::PostId> = None;
        assert_eq!(
            serde_json::to_value(&none).unwrap(),
            serde_json::Value::Null
        );
        let some = Some(post::PostId::new(5));
        assert_eq!(serde_json::to_value(&some).unwrap(), serde_json::json!(5));
    }
}

/// `Id<E, String>` accepts a `String` payload and survives a serde
/// round-trip. The phantom entity here is `user::Entity`, only the
/// inner-`T` behaviour is under test.
#[cfg(feature = "with-json")]
#[test]
fn string_typed_id_round_trips_through_serde() {
    type StringPk = sea_orm::Id<user::Entity, String>;
    let id: StringPk = sea_orm::Id::new("abc-xyz".to_string());
    let v = serde_json::to_value(&id).unwrap();
    assert_eq!(v, serde_json::json!("abc-xyz"));
    let back: StringPk = serde_json::from_value(v).unwrap();
    assert_eq!(back, sea_orm::Id::new("abc-xyz".to_string()));
}
