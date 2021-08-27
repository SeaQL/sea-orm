use rocket::fairing::{self, AdHoc};
use rocket::response::status::Created;
use rocket::serde::json::Json;
use rocket::{futures, Build, Rocket};

use rocket_db_pools::{sqlx, Connection, Database};

use sea_orm::entity::*;
use sea_orm::{DatabaseBackend, Statement};

mod setup;

#[derive(Database, Debug)]
#[database("blog")]
struct Db(sea_orm::Database);

type Result<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;

mod post;
pub use post::Entity as Post;

#[post("/", data = "<post>")]
async fn create(
    conn: Connection<Db>,
    post: Json<post::Model>,
) -> Result<Created<Json<post::Model>>> {
    let _post = post::ActiveModel {
        title: Set(post.title.to_owned()),
        text: Set(post.text.to_owned()),
        ..Default::default()
    }
    .save(&conn)
    .await
    .expect("could not insert post");

    Ok(Created::new("/").body(post))
}

#[get("/")]
async fn list(conn: Connection<Db>) -> Result<Json<Vec<i64>>> {
    let ids = Post::find()
        .all(&conn)
        .await
        .expect("could not retrieve posts")
        .into_iter()
        .map(|record| record.id.unwrap())
        .collect::<Vec<_>>();

    Ok(Json(ids))
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
async fn delete(conn: Connection<Db>, id: i64) -> Result<Option<()>> {
    let post: post::ActiveModel = Post::find_by_id(id)
        .one(&conn)
        .await
        .unwrap()
        .unwrap()
        .into();
    let result = post.delete(&conn).await.unwrap();

    Ok((result.rows_affected == 1).then(|| ()))
}

#[delete("/")]
async fn destroy(conn: Connection<Db>) -> Result<()> {
    let _result = Post::delete_many().exec(&conn).await.unwrap();
    Ok(())
}

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    let con = sea_orm::Database::connect("mysql://root:@localhost/rocket_example")
        .await
        .unwrap();
    let create_post_table = con
        .execute(Statement::from_string(
            DatabaseBackend::MySql,
            r#"
            CREATE TABLE posts (
                id int NOT NULL AUTO_INCREMENT,
                title VARCHAR(255) NOT NULL,
                text VARCHAR(255) NOT NULL,
                PRIMARY KEY (id)
            )"#
            .to_owned(),
        ))
        .await;
    println!("create_post_table: {:#?}", create_post_table);
    Ok(rocket)
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("SQLx Stage", |rocket| async {
        rocket
            .attach(Db::init())
            .attach(AdHoc::try_on_ignite("SQLx Migrations", run_migrations))
            .mount("/sqlx", routes![create, delete, destroy, list, read,])
    })
}
