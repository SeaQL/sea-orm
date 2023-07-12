pub mod common;
pub use common::{features::*, setup::*, TestContext};
use sea_orm::entity::prelude::*;
use sea_orm_macros::DeriveIden;

#[derive(DeriveIden)]
pub enum Class {
    Id,
    Title,
    Text,
}

#[derive(DeriveIden)]
pub enum Book {
    Id,
    #[sea_orm(iden = "turtle")]
    Title,
    #[sea_orm(iden = "TeXt")]
    Text,
    #[sea_orm(iden = "ty_pe")]
    Type,
}

#[derive(DeriveIden)]
struct Glyph;

#[derive(DeriveIden)]
#[sea_orm(iden = "weRd")]
struct Word;

#[test]
fn main() -> Result<(), DbErr> {
    assert_eq!(Class::Id.to_string(), "id");
    assert_eq!(Class::Title.to_string(), "title");
    assert_eq!(Class::Text.to_string(), "text");

    assert_eq!(Book::Id.to_string(), "id");
    assert_eq!(Book::Title.to_string(), "turtle");
    assert_eq!(Book::Text.to_string(), "TeXt");
    assert_eq!(Book::Type.to_string(), "ty_pe");

    assert_eq!(Glyph.to_string(), "glyph");

    assert_eq!(Word.to_string(), "weRd");
    Ok(())
}
