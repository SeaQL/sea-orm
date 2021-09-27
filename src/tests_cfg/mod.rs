//! Configurations for test cases and examples. Not intended for actual use.

pub mod cake;
pub mod cake_expanded;
pub mod cake_filling;
pub mod cake_filling_price;
pub mod entity_linked;
pub mod filling;
pub mod fruit;
pub mod vendor;

pub use cake::Entity as Cake;
pub use cake_expanded::Entity as CakeExpanded;
pub use cake_filling::Entity as CakeFilling;
pub use cake_filling_price::Entity as CakeFillingPrice;
pub use filling::Entity as Filling;
pub use fruit::Entity as Fruit;
pub use vendor::Entity as Vendor;
