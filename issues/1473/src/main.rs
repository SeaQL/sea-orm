use sea_orm::{DeriveIden, Iden};

#[derive(DeriveIden)]
enum Character {
    Table,
    Id,
}

#[derive(DeriveIden)]
struct Glyph;

fn main() {
    assert_eq!(Character::Table.to_string(), "character");
    assert_eq!(Character::Id.to_string(), "id");
    assert_eq!(Glyph.to_string(), "glyph");
}
