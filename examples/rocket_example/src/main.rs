#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_db_pools;

mod sqlx;

#[launch]
fn rocket() -> _ {
    rocket::build().attach(sqlx::stage())
}
