use std::env;

use entity::post;
use migration::{Migrator, MigratorTrait};
use salvo::extra::affix;
use salvo::extra::serve_static::DirHandler;
use salvo::prelude::*;
use salvo::writer::Text;
use salvo_example_core::{
    sea_orm::{Database, DatabaseConnection},
    Mutation, Query,
};
use tera::Tera;

const DEFAULT_POSTS_PER_PAGE: u64 = 5;
type Result<T> = std::result::Result<T, StatusError>;

#[derive(Debug, Clone)]
struct AppState {
    templates: tera::Tera,
    conn: DatabaseConnection,
}

#[handler]
async fn create(req: &mut Request, depot: &mut Depot, res: &mut Response) -> Result<()> {
    let state = depot
        .obtain::<AppState>()
        .ok_or_else(StatusError::internal_server_error)?;
    let conn = &state.conn;

    let form = req
        .extract_form::<post::Model>()
        .await
        .map_err(|_| StatusError::bad_request())?;

    Mutation::create_post(conn, form)
        .await
        .map_err(|_| StatusError::internal_server_error())?;

    res.redirect_found("/");
    Ok(())
}

#[handler]
async fn list(req: &mut Request, depot: &mut Depot) -> Result<Text<String>> {
    let state = depot
        .obtain::<AppState>()
        .ok_or_else(StatusError::internal_server_error)?;
    let conn = &state.conn;

    let page = req.query("page").unwrap_or(1);
    let posts_per_page = req
        .query("posts_per_page")
        .unwrap_or(DEFAULT_POSTS_PER_PAGE);

    let (posts, num_pages) = Query::find_posts_in_page(conn, page, posts_per_page)
        .await
        .map_err(|_| StatusError::internal_server_error())?;

    let mut ctx = tera::Context::new();
    ctx.insert("posts", &posts);
    ctx.insert("page", &page);
    ctx.insert("posts_per_page", &posts_per_page);
    ctx.insert("num_pages", &num_pages);

    let body = state
        .templates
        .render("index.html.tera", &ctx)
        .map_err(|_| StatusError::internal_server_error())?;
    Ok(Text::Html(body))
}

#[handler]
async fn new(depot: &mut Depot) -> Result<Text<String>> {
    let state = depot
        .obtain::<AppState>()
        .ok_or_else(StatusError::internal_server_error)?;
    let ctx = tera::Context::new();
    let body = state
        .templates
        .render("new.html.tera", &ctx)
        .map_err(|_| StatusError::internal_server_error())?;
    Ok(Text::Html(body))
}

#[handler]
async fn edit(req: &mut Request, depot: &mut Depot) -> Result<Text<String>> {
    let state = depot
        .obtain::<AppState>()
        .ok_or_else(StatusError::internal_server_error)?;
    let conn = &state.conn;
    let id = req.param::<i32>("id").unwrap_or_default();

    let post: post::Model = Query::find_post_by_id(conn, id)
        .await
        .map_err(|_| StatusError::internal_server_error())?
        .ok_or_else(StatusError::not_found)?;

    let mut ctx = tera::Context::new();
    ctx.insert("post", &post);

    let body = state
        .templates
        .render("edit.html.tera", &ctx)
        .map_err(|_| StatusError::internal_server_error())?;
    Ok(Text::Html(body))
}

#[handler]
async fn update(req: &mut Request, depot: &mut Depot, res: &mut Response) -> Result<()> {
    let state = depot
        .obtain::<AppState>()
        .ok_or_else(StatusError::internal_server_error)?;
    let conn = &state.conn;
    let id = req.param::<i32>("id").unwrap_or_default();
    let form = req
        .extract_form::<post::Model>()
        .await
        .map_err(|_| StatusError::bad_request())?;

    Mutation::update_post_by_id(conn, id, form)
        .await
        .map_err(|_| StatusError::internal_server_error())?;

    res.redirect_found("/");
    Ok(())
}

#[handler]
async fn delete(req: &mut Request, depot: &mut Depot, res: &mut Response) -> Result<()> {
    let state = depot
        .obtain::<AppState>()
        .ok_or_else(StatusError::internal_server_error)?;
    let conn = &state.conn;
    let id = req.param::<i32>("id").unwrap_or_default();

    Mutation::delete_post(conn, id)
        .await
        .map_err(|_| StatusError::internal_server_error())?;

    res.redirect_found("/");
    Ok(())
}

#[tokio::main]
pub async fn main() {
    std::env::set_var("RUST_LOG", "debug");
    tracing_subscriber::fmt::init();

    // get env vars
    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let server_url = format!("{}:{}", host, port);

    // create post table if not exists
    let conn = Database::connect(&db_url).await.unwrap();
    Migrator::up(&conn, None).await.unwrap();
    let templates = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
    let state = AppState { templates, conn };

    println!("Starting server at {}", server_url);

    let router = Router::new()
        .hoop(affix::inject(state))
        .post(create)
        .get(list)
        .push(Router::with_path("new").get(new))
        .push(Router::with_path("<id>").get(edit).post(update))
        .push(Router::with_path("delete/<id>").post(delete))
        .push(
            Router::with_path("static/<**>").get(DirHandler::new(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/static"
            ))),
        );

    Server::new(TcpListener::bind(&format!("{}:{}", host, port)))
        .serve(router)
        .await;
}
