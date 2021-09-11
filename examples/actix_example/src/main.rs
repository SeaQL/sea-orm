use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

// use std::collections::HashMap;

// use actix_http::{body::Body, Response};
// use actix_web::dev::ServiceResponse;
// use actix_web::http::StatusCode;
// use actix_web::middleware::errhandlers::{ErrorHandlerResponse, ErrorHandlers};
// use actix_web::{error, middleware, web, App, Error, HttpResponse, HttpServer, Result};
// use tera::Tera;

// // store tera template in application state
// async fn index(
//     tmpl: web::Data<tera::Tera>,
//     query: web::Query<HashMap<String, String>>,
// ) -> Result<HttpResponse, Error> {
//     let s = if let Some(name) = query.get("name") {
//         // submitted form
//         let mut ctx = tera::Context::new();
//         ctx.insert("name", &name.to_owned());
//         ctx.insert("text", &"Welcome!".to_owned());
//         tmpl.render("user.html", &ctx)
//             .map_err(|_| error::ErrorInternalServerError("Template error"))?
//     } else {
//         tmpl.render("index.html", &tera::Context::new())
//             .map_err(|_| error::ErrorInternalServerError("Template error"))?
//     };
//     Ok(HttpResponse::Ok().content_type("text/html").body(s))
// }

// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     HttpServer::new(|| {
//         App::new()
//             .service(hello)
//             .service(echo)
//             .route("/hey", web::get().to(manual_hello))
//     })
//     .bind("127.0.0.1:8080")?
//     .run()
//     .await
// }


// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     std::env::set_var("RUST_LOG", "actix_web=info");
//     env_logger::init();

//     println!("Listening on: 127.0.0.1:8080, open browser and visit have a try!");
//     HttpServer::new(|| {
//         let tera =
//             Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();

//         App::new()
//             .data(tera)
//             .wrap(middleware::Logger::default()) // enable logger
//             .service(web::resource("/").route(web::get().to(index)))
//             .service(web::scope("").wrap(error_handlers()))
//     })
//     .bind("127.0.0.1:8080")?
//     .run()
//     .await
// }

// // Custom error handlers, to return HTML responses when an error occurs.
// fn error_handlers() -> ErrorHandlers<Body> {
//     ErrorHandlers::new().handler(StatusCode::NOT_FOUND, not_found)
// }

// // Error handler for a 404 Page not found error.
// fn not_found<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
//     let response = get_error_response(&res, "Page not found");
//     Ok(ErrorHandlerResponse::Response(
//         res.into_response(response.into_body()),
//     ))
// }

// // Generic error handler.
// fn get_error_response<B>(res: &ServiceResponse<B>, error: &str) -> Response<Body> {
//     let request = res.request();

//     // Provide a fallback to a simple plain text response in case an error occurs during the
//     // rendering of the error page.
//     let fallback = |e: &str| {
//         Response::build(res.status())
//             .content_type("text/plain")
//             .body(e.to_string())
//     };

//     let tera = request.app_data::<web::Data<Tera>>().map(|t| t.get_ref());
//     match tera {
//         Some(tera) => {
//             let mut context = tera::Context::new();
//             context.insert("error", error);
//             context.insert("status_code", res.status().as_str());
//             let body = tera.render("error.html", &context);

//             match body {
//                 Ok(body) => Response::build(res.status())
//                     .content_type("text/html")
//                     .body(body),
//                 Err(_) => fallback(error),
//             }
//         }
//         None => fallback(error),
//     }
// }

// -----------------------------
// #[macro_use]
// extern crate rocket;

// use rocket::fairing::{self, AdHoc};
// use rocket::form::{Context, Form};
// use rocket::fs::{relative, FileServer};
// use rocket::request::FlashMessage;
// use rocket::response::{Flash, Redirect};
// use rocket::{Build, Request, Rocket};
// use rocket_db_pools::{sqlx, Connection, Database};
// use rocket_dyn_templates::{context, Template};

// use sea_orm::entity::*;

// mod pool;
// use pool::RocketDbPool;

// mod setup;

// #[derive(Database, Debug)]
// #[database("rocket_example")]
// struct Db(RocketDbPool);

// type Result<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;

// mod post;
// pub use post::Entity as Post;

// const DEFAULT_POSTS_PER_PAGE: usize = 25;

// #[get("/new")]
// fn new() -> Template {
//     Template::render("new", &Context::default())
// }

// #[post("/", data = "<post_form>")]
// async fn create(conn: Connection<Db>, post_form: Form<post::Model>) -> Flash<Redirect> {
//     let form = post_form.into_inner();

//     post::ActiveModel {
//         title: Set(form.title.to_owned()),
//         text: Set(form.text.to_owned()),
//         ..Default::default()
//     }
//     .save(&conn)
//     .await
//     .expect("could not insert post");

//     Flash::success(Redirect::to("/"), "Post successfully added.")
// }

// #[post("/<id>", data = "<post_form>")]
// async fn update(conn: Connection<Db>, id: i32, post_form: Form<post::Model>) -> Flash<Redirect> {
//     let post: post::ActiveModel = Post::find_by_id(id)
//         .one(&conn)
//         .await
//         .unwrap()
//         .unwrap()
//         .into();

//     let form = post_form.into_inner();

//     post::ActiveModel {
//         id: post.id,
//         title: Set(form.title.to_owned()),
//         text: Set(form.text.to_owned()),
//     }
//     .save(&conn)
//     .await
//     .expect("could not edit post");

//     Flash::success(Redirect::to("/"), "Post successfully edited.")
// }

// #[get("/?<page>&<posts_per_page>")]
// async fn list(
//     conn: Connection<Db>,
//     posts_per_page: Option<usize>,
//     page: Option<usize>,
//     flash: Option<FlashMessage<'_>>,
// ) -> Template {
//     let page = page.unwrap_or(0);
//     let posts_per_page = posts_per_page.unwrap_or(DEFAULT_POSTS_PER_PAGE);
//     let paginator = Post::find().paginate(&conn, posts_per_page);
//     let num_pages = paginator.num_pages().await.ok().unwrap();

//     let posts = paginator
//         .fetch_page(page)
//         .await
//         .expect("could not retrieve posts");

//     let flash = flash.map(FlashMessage::into_inner);

//     Template::render(
//         "index",
//         context! {
//             posts: posts,
//             flash: flash,
//             page: page,
//             num_pages: num_pages,
//         },
//     )
// }

// #[get("/<id>")]
// async fn edit(conn: Connection<Db>, id: i32) -> Template {
//     let post: Option<post::Model> = Post::find_by_id(id)
//         .one(&conn)
//         .await
//         .expect("could not find post");

//     Template::render(
//         "edit",
//         context! {
//             post: post,
//         },
//     )
// }

// #[delete("/<id>")]
// async fn delete(conn: Connection<Db>, id: i32) -> Flash<Redirect> {
//     let post: post::ActiveModel = Post::find_by_id(id)
//         .one(&conn)
//         .await
//         .unwrap()
//         .unwrap()
//         .into();

//     post.delete(&conn).await.unwrap();

//     Flash::success(Redirect::to("/"), "Post successfully deleted.")
// }

// #[delete("/")]
// async fn destroy(conn: Connection<Db>) -> Result<()> {
//     Post::delete_many().exec(&conn).await.unwrap();
//     Ok(())
// }

// #[catch(404)]
// pub fn not_found(req: &Request<'_>) -> Template {
//     Template::render(
//         "error/404",
//         context! {
//             uri: req.uri()
//         },
//     )
// }

// async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
//     let db_url = Db::fetch(&rocket).unwrap().db_url.clone();
//     let conn = sea_orm::Database::connect(&db_url).await.unwrap();
//     let _ = setup::create_post_table(&conn).await;
//     Ok(rocket)
// }

// #[launch]
// fn rocket() -> _ {
//     rocket::build()
//         .attach(Db::init())
//         .attach(AdHoc::try_on_ignite("Migrations", run_migrations))
//         .mount("/", FileServer::from(relative!("/static")))
//         .mount(
//             "/",
//             routes![new, create, delete, destroy, list, edit, update],
//         )
//         .register("/", catchers![not_found])
//         .attach(Template::fairing())
// }
