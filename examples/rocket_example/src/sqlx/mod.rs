use rocket::fairing::AdHoc;
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
            .mount("/sqlx", routes![create, delete, destroy, list, read,])
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
