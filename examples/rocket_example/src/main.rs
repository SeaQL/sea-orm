#[macro_use]
extern crate rocket;

mod sqlx;

#[launch]
fn rocket() -> _ {
    rocket::build().attach(sqlx::stage())
}
