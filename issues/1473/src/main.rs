use sea_orm::Iden;

#[derive(Iden)]
enum Character {
    Table,
    Id,
}

#[derive(Iden)]
struct Glyph;

fn main() {
    assert_eq!(Character::Table.to_string(), "character");
    assert_eq!(Character::Id.to_string(), "id");

    assert_eq!(Glyph.to_string(), "glyph");
}
