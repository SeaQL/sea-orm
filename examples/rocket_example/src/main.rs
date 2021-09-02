#[macro_use]
extern crate rocket;

use rocket::fairing::{self, AdHoc};
use rocket::form::{Context, Form};
use rocket::fs::{relative, FileServer};
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket::serde::json::Json;
use rocket::{Build, Request, Rocket};
use rocket_db_pools::{sqlx, Connection, Database};
use rocket_dyn_templates::{context, Template};

use sea_orm::entity::*;
use sea_orm::RocketDbPool;

mod setup;

#[derive(Database, Debug)]
#[database("rocket_example")]
struct Db(RocketDbPool);

type Result<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;

mod post;
pub use post::Entity as Post;

#[get("/new")]
fn new() -> Template {
    Template::render("new", &Context::default())
}

#[post("/", data = "<post_form>")]
async fn create(conn: Connection<Db>, post_form: Form<post::Model>) -> Flash<Redirect> {
    let post = post_form.into_inner();

    let _post = post::ActiveModel {
        title: Set(post.title.to_owned()),
        text: Set(post.text.to_owned()),
        ..Default::default()
    }
    .save(&conn)
    .await
    .expect("could not insert post");

    Flash::success(Redirect::to("/"), "Post successfully added.")
}

#[get("/")]
async fn list(conn: Connection<Db>, flash: Option<FlashMessage<'_>>) -> Template {
    let posts = Post::find()
        .all(&conn)
        .await
        .expect("could not retrieve posts")
        .into_iter()
        .collect::<Vec<_>>();
    let flash = flash.map(FlashMessage::into_inner);

    Template::render(
        "index",
        context! {
            posts: posts,
            flash: flash,
        },
    )
}

#[get("/<id>")]
async fn read(conn: Connection<Db>, id: i64) -> Option<Json<post::Model>> {
    let post: Option<post::Model> = Post::find_by_id(id)
        .one(&conn)
        .await
        .expect("could not find post");

    match post {
        None => None,
        Some(post) => Some(Json(post)),
    }
}

#[delete("/<id>")]
async fn delete(conn: Connection<Db>, id: i32) -> Flash<Redirect> {
    let post: post::ActiveModel = Post::find_by_id(id)
        .one(&conn)
        .await
        .unwrap()
        .unwrap()
        .into();
    let _result = post.delete(&conn).await.unwrap();

    Flash::success(Redirect::to("/"), "Post successfully deleted.")
}

#[delete("/")]
async fn destroy(conn: Connection<Db>) -> Result<()> {
    let _result = Post::delete_many().exec(&conn).await.unwrap();
    Ok(())
}

#[catch(404)]
pub fn not_found(req: &Request<'_>) -> Template {
    Template::render(
        "error/404",
        context! {
            uri: req.uri()
        },
    )
}

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    let db_url = Db::fetch(&rocket).unwrap().db_url.clone();
    let conn = sea_orm::Database::connect(&db_url).await.unwrap();
    let _create_post_table = setup::create_post_table(&conn).await;
    Ok(rocket)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Db::init())
        .attach(AdHoc::try_on_ignite("Migrations", run_migrations))
        .mount("/", FileServer::from(relative!("/static")))
        .mount("/", routes![new, create, delete, destroy, list, read,])
        .register("/", catchers![not_found])
        .attach(Template::fairing())
}
