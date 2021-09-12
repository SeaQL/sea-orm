// use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};


use actix_http::{body::Body, Response};
use actix_web::dev::ServiceResponse;
use actix_web::http::StatusCode;
use actix_web::middleware::errhandlers::{ErrorHandlerResponse, ErrorHandlers};
use actix_web::{error, middleware, web, App, Error, HttpResponse, HttpServer, Result};
use actix_files as fs;

use tera::Tera;

mod post;
pub use post::Entity as Post;


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

async fn list( tmpl: web::Data<tera::Tera>) -> Result<HttpResponse, Error> {
    let posts: Vec<post::Model> = vec!();
    let mut ctx = tera::Context::new();
    ctx.insert("posts", &posts);
    ctx.insert("page", &0);
    ctx.insert("num_pages", &1);

    let s = tmpl.render("index.html.tera", &ctx)
            .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    println!("Listening on: 127.0.0.1:8080");
    HttpServer::new(|| {
        let tera =
            Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
        App::new()
            .data(tera)
            .wrap(middleware::Logger::default()) // enable logger
            .service(fs::Files::new("/static", "./static").show_files_listing())
            .service(web::resource("/").route(web::get().to(list)))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
