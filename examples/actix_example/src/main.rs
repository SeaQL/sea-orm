// use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

use actix_files as fs;
use actix_http::{body::Body, Response};
use actix_web::dev::ServiceResponse;
use actix_web::http::StatusCode;
use actix_web::middleware::errhandlers::{ErrorHandlerResponse, ErrorHandlers};
use actix_web::{error, get, middleware, post, web, App, Error, HttpResponse, HttpServer, Result};

use tera::Tera;

mod post;
pub use post::Entity as Post;
use sea_orm::query::*;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;

mod setup;

struct AppState {
    db_url: String,
    templates: tera::Tera,
}

#[get("/")]
async fn list(data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let template = &data.templates;
    let conn = sea_orm::Database::connect(&data.db_url).await.unwrap();

    let posts = Post::find()
        .all(&conn)
        .await
        .expect("could not retrieve posts");

    let mut ctx = tera::Context::new();
    ctx.insert("posts", &posts);
    ctx.insert("page", &0);
    ctx.insert("num_pages", &1);

    let body = template
        .render("index.html.tera", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let db_url = "mysql://root:@localhost/rocket_example";
    let conn = sea_orm::Database::connect(&db_url).await.unwrap();
    let _ = setup::create_post_table(&conn).await;

    println!("Listening on: 127.0.0.1:8080");
    HttpServer::new(move || {
        let templates = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
        App::new()
            .data(AppState {
                db_url: db_url.to_owned(),
                templates: templates,
            })
            .wrap(middleware::Logger::default()) // enable logger
            .service(fs::Files::new("/static", "./static").show_files_listing())
            .configure(init) // init todo routes
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

// function that will be called on new Application to configure routes for this module
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(list);
}
