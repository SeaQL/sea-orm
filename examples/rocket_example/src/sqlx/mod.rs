use rocket::fairing::{self, AdHoc};
use rocket::response::status::Created;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{futures, Build, Rocket};

use rocket_db_pools::{sqlx, Connection, Database};

use futures::{future::TryFutureExt, stream::TryStreamExt};

// use post::*;
mod post;
pub use post::Entity as Post;

#[derive(Database)]
#[database("sqlx")]
struct Db(sqlx::SqlitePool);

type Result<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;

// #[derive(Debug, Clone, Deserialize, Serialize)]
// #[serde(crate = "rocket::serde")]
// struct Post {
//     #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
//     id: Option<i64>,
//     title: String,
//     text: String,
// }

// #[post("/", data = "<post>")]
// async fn create(mut db: Connection<Db>, post: Json<Post>) -> Result<Created<Json<Post>>> {
//     // There is no support for `RETURNING`.
//     sqlx::query!(
//         "INSERT INTO posts (title, text) VALUES (?, ?)",
//         post.title,
//         post.text
//     )
//     .execute(&mut *db)
//     .await?;

//     Ok(Created::new("/").body(post))
// }

#[get("/")]
async fn list(mut db: Connection<Db>) -> Result<Json<Vec<i64>>> {
    // let ids = sqlx::query!("SELECT id FROM posts")
    //     .fetch(&mut *db)
    //     .map_ok(|record| record.id)
    //     .try_collect::<Vec<_>>()
    //     .await?;
    let ids = vec![];
    Ok(Json(ids))
}

#[get("/<id>")]
async fn read(mut db: Connection<Db>, id: i64) -> Option<Json<Post>> {
    let post: Option<post::Model> = Post::find_by_id(id)
        .one(db)
        .await
        .expect("could not find baker");
    println!("post: {:#?}", post);

    // sqlx::query!("SELECT id, title, text FROM posts WHERE id = ?", id)
    //     .fetch_one(&mut *db)
    //     .map_ok(|r| {
    //         Json(Post {
    //             id: Some(r.id),
    //             title: r.title,
    //             text: r.text,
    //         })
    //     })
    //     .await
    //     .ok()

    None
}

// #[delete("/<id>")]
// async fn delete(mut db: Connection<Db>, id: i64) -> Result<Option<()>> {
//     let result = sqlx::query!("DELETE FROM posts WHERE id = ?", id)
//         .execute(&mut *db)
//         .await?;

//     Ok((result.rows_affected() == 1).then(|| ()))
// }

// #[delete("/")]
// async fn destroy(mut db: Connection<Db>) -> Result<()> {
//     sqlx::query!("DELETE FROM posts").execute(&mut *db).await?;

//     Ok(())
// }

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    match Db::fetch(&rocket) {
        Some(db) => match sqlx::migrate!("db/sqlx/migrations").run(&**db).await {
            Ok(_) => Ok(rocket),
            Err(e) => {
                error!("Failed to initialize SQLx database: {}", e);
                Err(rocket)
            }
        },
        None => Err(rocket),
    }
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("SQLx Stage", |rocket| async {
        rocket
            .attach(Db::init())
            .attach(AdHoc::try_on_ignite("SQLx Migrations", run_migrations))
            .mount("/sqlx", routes![list, read])
    })
}
