#![allow(unused_imports, dead_code)]

//! Configurations for test cases and examples. Not intended for actual use.

#[cfg(feature = "entity-registry")]
mod registry;

pub mod cake;
pub mod cake_compact;
pub mod cake_expanded;
pub mod cake_filling;
pub mod cake_filling_price;
pub mod entity_linked;
pub mod filling;
pub mod fruit;
pub mod indexes;
pub mod ingredient;
pub mod lunch_set;
pub mod lunch_set_expanded;
pub mod rust_keyword;
pub mod sea_orm_active_enums;
pub mod serde_rename;
pub mod vendor;

pub mod comment;
pub mod post;
pub mod post_tag;
pub mod profile;
pub mod tag;
pub mod user;

pub use cake::Entity as Cake;
pub use cake_filling::Entity as CakeFilling;
pub use cake_filling_price::Entity as CakeFillingPrice;
pub use filling::Entity as Filling;
pub use fruit::Entity as Fruit;
pub use lunch_set::Entity as LunchSet;
pub use lunch_set_expanded::Entity as LunchSetExpanded;
pub use rust_keyword::Entity as RustKeyword;
pub use vendor::Entity as Vendor;
