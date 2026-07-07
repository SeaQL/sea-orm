//! Runtime-typed entity API for working with schemas that are not known at
//! compile time.
//!
//! Where the rest of SeaORM is statically typed around generated `Entity` /
//! `Model` / `Column` types, this module exposes an [`Entity`], [`Column`],
//! and [`Model`] that carry their schema information at runtime.
//!
//! **Unstable.** The API in this module may change between minor versions.
#![allow(missing_docs)]

mod entity;
mod execute;
mod model;

pub use entity::*;
pub use execute::*;
pub use model::*;
