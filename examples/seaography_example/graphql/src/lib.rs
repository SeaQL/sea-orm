use sea_orm::prelude::*;

pub mod entities;
pub mod query_root;

pub struct OrmDataloader {
    pub db: DatabaseConnection,
}
