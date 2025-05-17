use sea_orm::{Set, tests_cfg::cake};

struct Cake {
    id: i32,
    name: String,
}

impl From<Cake> for cake::ActiveModel {
    fn from(value: Cake) -> Self {
        Self {
            id: Set(value.id),
            name: Set(value.name),
        }
    }
}
