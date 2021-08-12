#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_db_pools;

#[cfg(test)]
mod tests;

mod sqlx;

#[launch]
fn rocket() -> _ {
    rocket::build().attach(sqlx::stage())
}
