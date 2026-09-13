pub mod person;
pub mod post;

pub struct Entities;

impl sea_orm_migration::EntitySet for Entities {
    fn register(
        self,
        builder: sea_orm_migration::SchemaBuilder,
    ) -> sea_orm_migration::SchemaBuilder {
        builder.register(post::Entity).register(person::Entity)
    }
}
