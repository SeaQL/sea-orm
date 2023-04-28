use sea_orm::DeriveIden;

#[derive(DeriveIden)]
enum Posts {
    Table,
    Id,
    Title,
    Text,
}

fn main() {}
