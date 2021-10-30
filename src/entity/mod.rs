//! This modules contains types and traits for an  Entity, ActiveMode, Model, PrimaryKey, ForeignKey and Relations.
//!
//! // An Entity
//! A unit struct implements [EntityTrait](crate::EntityTrait) representing a table in the database.
//!
//! This trait contains the properties of an entity including
//!
//!     Table Name (implemented EntityName)
//!     Column (implemented ColumnTrait)
//!     Relation (implemented RelationTrait)
//!     Primary Key (implemented PrimaryKeyTrait and PrimaryKeyToColumn)
//!
//! This trait also provides an API for CRUD actions
//!
//!     Select: find, find_*
//!     Insert: insert, insert_*
//!     Update: update, update_*
//!     Delete: delete, delete_*
//!
//! #### Example for creating an Entity, Model and ActiveModel
//! ```
//! use sea_orm::entity::prelude::*;
//!
//! // Use [DeriveEntity] to derive the EntityTrait automatically
//! #[derive(Copy, Clone, Default, Debug, DeriveEntity)]
//! pub struct Entity;
//!
//! /// The [EntityName] describes the name of a table
//! impl EntityName for Entity {
//!     fn table_name(&self) -> &str {
//!         "cake"
//!     }
//! }
//!
//! // Create a Model for the Entity through [DeriveModel].
//! // The `Model` handles `READ` operations on a table in a database.
//! // The [DeriveActiveModel] creates a way to perform `CREATE` , `READ` and `UPDATE` operations
//! // in a database
//!
//! #[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel)]
//! pub struct Model {
//!     pub id: i32,
//!     pub name: Option<String> ,
//! }
//!
//! // Use the [DeriveColumn] to create a Column for an the table called Entity
//! // The [EnumIter] which creates a new type that iterates of the variants of a Column.
//! #[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
//! pub enum Column {
//!     Id,
//!     Name,
//! }
//!
//! // Create a PrimaryKey for the Entity using the [PrimaryKeyTrait]
//! // The [EnumIter] which creates a new type that iterates of the variants of a PrimaryKey.
//! #[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
//! pub enum PrimaryKey {
//!     Id,
//! }
//!
//! // Or implement the [PrimaryKeyTrait] manually instead of using the macro [DerivePrimaryKey]
//! impl PrimaryKeyTrait for PrimaryKey {
//!     type ValueType = i32;
//!
//!     fn auto_increment() -> bool {
//!         true
//!     }
//! }
//!
//! // Create a Relation for the Entity
//! #[derive(Copy, Clone, Debug, EnumIter)]
//! #[sea_orm(
//!        // The relation belongs to `Entity` type
//!        belongs_to = "Entity",
//!        from = "Column::FruitId",
//!        to = "Column::Id"
//!    )]
//! pub enum Relation {
//!     Fruit,
//! }
//!
//! // Create the properties of a Column in an Entity ensuring that calling
//! // Column::def() yields a Column definition as defined in [ColumnDef]
//! impl ColumnTrait for Column {
//!     type EntityName = Entity;
//!     fn def(&self) -> ColumnDef {
//!         match self {
//!             Self::Id => ColumnType::Integer.def(),
//!             Self::Name => ColumnType::Text.def().null(),
//!         }
//!     }
//! }
//!
//! // Implement the set of constraints for creating a Relation as defined in the [RelationTrait]
//! impl RelationTrait for Relation {
//!     fn def(&self) -> RelationDef {
//!         match self {
//!             Self::Fruit => Entity::has_many(super::fruit::Entity).into(),
//!         }
//!     }
//! }
//!
//! impl Related<fruit::Entity> for Entity {
//!     fn to() -> RelationDef {
//!         Relation::Fruit.def()
//!     }
//! }
//!
//! impl Related<super::filling::Entity> for Entity {
//!     fn to() -> RelationDef {
//!         super::cake_filling::Relation::Filling.def()
//!     }
//!     fn via() -> Option<RelationDef> {
//!         Some(super::cake_filling::Relation::Cake.def().rev())
//!     }
//! }
//!
//! // Implement user defined operations for CREATE, UPDATE and DELETE operations
//! // to create an ActiveModel using the [ActiveModelBehavior]
//! impl ActiveModelBehavior for ActiveModel {}
//! ```

mod active_model;
mod base_entity;
mod column;
mod identity;
mod link;
mod model;
/// Re-export common types from the entity
pub mod prelude;
mod primary_key;
mod relation;

pub use active_model::*;
pub use base_entity::*;
pub use column::*;
pub use identity::*;
pub use link::*;
pub use model::*;
// pub use prelude::*;
pub use primary_key::*;
pub use relation::*;
