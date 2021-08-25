use rocket::fairing::{self, AdHoc};
use rocket::response::status::Created;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{futures, Build, Rocket};

use rocket_db_pools::{sqlx, Connection, Database};

use futures::{future::TryFutureExt, stream::TryStreamExt};
use sea_orm::entity::*;
use sea_orm::{
    DatabaseBackend, QueryFilter, SqlxSqliteConnector, SqlxSqlitePoolConnection, Statement,
};

mod setup;

#[derive(Database, Debug)]
#[database("blog")]
struct Db(sea_orm::Database);

type Result<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;

// use post::*;
mod post;
pub use post::Entity as Post;
use sea_orm::DatabaseConnection;

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
async fn list(mut con: Connection<Db>) -> Result<Json<Vec<i64>>> {
    // let ids = sqlx::query!("SELECT id FROM posts")
    //     .fetch(&mut *db)
    //     .map_ok(|record| record.id)
    //     .try_collect::<Vec<_>>()
    //     .await?;
    // // let ids: Vec<i64> = vec![];

    // let ids = sqlx::query(
    //     r#"
    //         SELECT id FROM posts
    //     "#,
    // )
    // .execute(&mut *db)
    // .await?;
    // // .map_ok(|record| record.id);
    // // .try_collect::<Vec<_>>();
    // println!("ids: {:#?}", ids);

    // let ids: Vec<i64> = vec![];
    // Ok(Json(ids))

    // let mut conn = db.acquire().await?;
    // println!("conn: {:#?}", conn);

    // let ids = sqlx::query("SELECT id FROM posts")
    //     .fetch(&mut *db)
    //     .map_ok(|record| record.id)
    //     .try_collect::<Vec<_>>()
    //     .await?;

    // Ok(Json(ids))

    // let recs = sqlx::query(
    //     r#"
    //     SELECT id FROM posts
    //     "#,
    // )
    // .fetch_all(&mut *db)
    // .await?;
    // let ids: Vec<i64> = recs.into();

    // println!("recs: {:#?}", ids);
    // println!("db: {:#?}", &*db);
    // let res = db
    //     .execute(Statement::from_string(
    //         DatabaseBackend::Sqlite,
    //         "SELECT * from posts".to_owned(),
    //     ))
    //     .await;
    // println!("res: {:#?}", res);

    let all_posts = con
        .query_all(Statement::from_string(
            DatabaseBackend::MySql,
            "select * from posts;".to_owned(),
        ))
        .await
        .unwrap();
    for post in all_posts.into_iter() {
        // let p = Post::from_raw_query_result(post);
        println!(
            "p: {:#?}",
            sea_orm::JsonValue::from_query_result(&post, "").unwrap()
        );
    }

    // let con = SqlxSqliteConnector::from_sqlx_sqlite_pool(db);
    // let posts = Post::find().all(&con).await.unwrap();
    // assert_eq!(posts.len(), 0);

    let ids: Vec<i64> = vec![];
    Ok(Json(ids))
}

#[get("/<id>")]
async fn read(mut db: Connection<Db>, id: i64) -> Option<Json<Post>> {
    // let post: Option<post::Model> = Post::find_by_id(id)
    //     .one(db)
    //     .await
    //     .expect("could not find baker");
    // println!("post: {:#?}", post);

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

// async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
//     use crate::rocket_db_pools::Pool;
//     // match Db::fetch(&rocket) {
//     //     Some(db) => match sqlx::migrate!("db/sqlx/migrations").run(&**db).await {
//     //         Ok(_) => Ok(rocket),
//     //         Err(e) => {
//     //             error!("Failed to initialize SQLx database: {}", e);
//     //             Err(rocket)
//     //         }
//     //     },
//     //     None => Err(rocket),
//     // }
//     // let conn = Db::get(&rocket).await.expect("database connection");

//     match Db::fetch(&rocket) {
//         Some(db) => match setup::create_post_table(db.get().await().expect("database connection")).await {
//             Ok(_) => {
//                 println!("rocket: {:#?}", rocket);

//                 Ok(rocket)
//             }
//             Err(e) => {
//                 error!("Failed to initialize SQLx database: {}", e);
//                 Err(rocket)
//             }
//         },
//         None => Err(rocket),
//     }
//     // Ok(rocket)
// }

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("SQLx Stage", |rocket| async {
        rocket
            .attach(Db::init())
            .attach(AdHoc::try_on_ignite("Create init post", |rocket| async {
                let con = sea_orm::Database::connect("mysql://root:@localhost/rocket_example")
                    .await
                    .unwrap();
                // let res = sqlx::query(
                //     r#"
                //     CREATE TABLE posts (
                //         id INTEGER PRIMARY KEY AUTOINCREMENT,
                //         title VARCHAR NOT NULL,
                //         text VARCHAR NOT NULL,
                //         published BOOLEAN NOT NULL DEFAULT 0
                //     )"#,
                // )
                // .execute(&**db)
                // .await;
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

                let create_post = con
                    .execute(Statement::from_string(
                        DatabaseBackend::MySql,
                        "INSERT INTO posts (title, text) VALUES ('a post', 'content of a post')"
                            .to_owned(),
                    ))
                    .await;
                println!("create_post: {:#?}", create_post);

                // println!("all_posts: {:#?}", all_posts);

                // let res2 = sqlx::query(
                //     r#"
                //     INSERT INTO posts (title, text) VALUES ('a post', 'content of a post')
                //     "#,
                // )
                // .execute(&**db)
                // .await;
                // println!("res2: {:#?}", res2);

                // Db::fetch(&rocket)
                //     .run(|db| {
                //         sqlx::query("DELETE FROM table").execute(&pool).await;

                //         // conn.execute(
                //         //     r#"
                //         //     CREATE TABLE posts (
                //         //         id INTEGER PRIMARY KEY AUTOINCREMENT,
                //         //         title VARCHAR NOT NULL,
                //         //         text VARCHAR NOT NULL,
                //         //         published BOOLEAN NOT NULL DEFAULT 0
                //         //     )"#,
                //         //     params![],
                //         // )
                //     })
                //     .await
                //     .expect("can init rusqlite DB");
                Ok(rocket)

                // match Db::fetch(&rocket) {
                //     Some(db) => {
                //         println!("db: {:#?}", db);
                //         println!("&**db: {:#?}", &**db);

                //         Ok(rocket)
                //     }
                //     None => Err(rocket),
                // }
            }))
            .mount("/sqlx", routes![list, read])
    })
}

// pub async fn create_post(db: &DbConn) {
//     let post = post::ActiveModel {
//         title: Set("Post One".to_owned()),
//         text: Set("post content 1".to_owned()),
//         ..Default::default()
//     }
//     .save(db)
//     .await
//     .expect("could not insert post");
// }
