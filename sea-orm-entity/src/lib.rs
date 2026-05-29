pub mod cli;
pub mod codegen;
pub mod filter;
pub mod fs;
pub mod summary;

pub use sea_orm::schema::SchemaBuilder;

/// Trait for a set of entities to be registered into a [`SchemaBuilder`].
///
/// Implement this on a unit struct in your entity crate:
///
/// ```rust,ignore
/// pub struct Entities;
///
/// impl sea_orm_entity::EntitySet for Entities {
///     fn register(self, builder: sea_orm_entity::SchemaBuilder) -> sea_orm_entity::SchemaBuilder {
///         builder
///             .register(user::Entity)
///             .register(post::Entity)
///     }
/// }
/// ```
pub trait EntitySet {
    fn register(self, builder: SchemaBuilder) -> SchemaBuilder;
}
