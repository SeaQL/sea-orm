use std::env;

use entity::post;
use migration::{Migrator, MigratorTrait};
use poem::endpoint::StaticFilesEndpoint;
use poem::error::InternalServerError;
use poem::http::StatusCode;
use poem::listener::TcpListener;
use poem::web::{Data, Form, Html, Path, Query};
use poem::{get, handler, post, EndpointExt, Error, IntoResponse, Result, Route, Server};
use poem_example_service::{
    sea_orm::{Database, DatabaseConnection},
    Mutation as MutationCore, Query as QueryCore,
};
use serde::Deserialize;
use tera::Tera;

const DEFAULT_POSTS_PER_PAGE: u64 = 5;

#[derive(Debug, Clone)]
struct AppState {
    templates: tera::Tera,
    conn: DatabaseConnection,
}

#[derive(Deserialize)]
struct Params {
    page: Option<u64>,
    posts_per_page: Option<u64>,
}

#[handler]
async fn create(state: Data<&AppState>, form: Form<post::Model>) -> Result<impl IntoResponse> {
    let form = form.0;
    let conn = &state.conn;

    MutationCore::create_post(conn, form)
        .await
        .map_err(InternalServerError)?;

    Ok(StatusCode::FOUND.with_header("location", "/"))
}

#[handler]
async fn list(state: Data<&AppState>, Query(params): Query<Params>) -> Result<impl IntoResponse> {
    let conn = &state.conn;
    let page = params.page.unwrap_or(1);
    let posts_per_page = params.posts_per_page.unwrap_or(DEFAULT_POSTS_PER_PAGE);

    let (posts, num_pages) = QueryCore::find_posts_in_page(conn, page, posts_per_page)
        .await
        .map_err(InternalServerError)?;

    let mut ctx = tera::Context::new();
    ctx.insert("posts", &posts);
    ctx.insert("page", &page);
    ctx.insert("posts_per_page", &posts_per_page);
    ctx.insert("num_pages", &num_pages);

    let body = state
        .templates
        .render("index.html.tera", &ctx)
        .map_err(InternalServerError)?;
    Ok(Html(body))
}

#[handler]
async fn new(state: Data<&AppState>) -> Result<impl IntoResponse> {
    let ctx = tera::Context::new();
    let body = state
        .templates
        .render("new.html.tera", &ctx)
        .map_err(InternalServerError)?;
    Ok(Html(body))
}

#[handler]
async fn edit(state: Data<&AppState>, Path(id): Path<i32>) -> Result<impl IntoResponse> {
    let conn = &state.conn;

    let post: post::Model = QueryCore::find_post_by_id(conn, id)
        .await
        .map_err(InternalServerError)?
        .ok_or_else(|| Error::from_status(StatusCode::NOT_FOUND))?;

    let mut ctx = tera::Context::new();
    ctx.insert("post", &post);

    let body = state
        .templates
        .render("edit.html.tera", &ctx)
        .map_err(InternalServerError)?;
    Ok(Html(body))
}

#[handler]
async fn update(
    state: Data<&AppState>,
    Path(id): Path<i32>,
    form: Form<post::Model>,
) -> Result<impl IntoResponse> {
    let conn = &state.conn;
    let form = form.0;

    MutationCore::update_post_by_id(conn, id, form)
        .await
        .map_err(InternalServerError)?;

    Ok(StatusCode::FOUND.with_header("location", "/"))
}

#[handler]
async fn delete(state: Data<&AppState>, Path(id): Path<i32>) -> Result<impl IntoResponse> {
    let conn = &state.conn;

    MutationCore::delete_post(conn, id)
        .await
        .map_err(InternalServerError)?;

    Ok(StatusCode::FOUND.with_header("location", "/"))
}

#[tokio::main]
async fn start() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    tracing_subscriber::fmt::init();

    // get env vars
    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let server_url = format!("{host}:{port}");

    // create post table if not exists
    let conn = Database::connect(&db_url).await.unwrap();
    Migrator::up(&conn, None).await.unwrap();
    let templates = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
    let state = AppState { templates, conn };

    println!("Starting server at {server_url}");

    let app = Route::new()
        .at("/", post(create).get(list))
        .at("/new", new)
        .at("/:id", get(edit).post(update))
        .at("/delete/:id", post(delete))
        .nest(
            "/static",
            StaticFilesEndpoint::new(concat!(env!("CARGO_MANIFEST_DIR"), "/static")),
        )
        .data(state);
    let server = Server::new(TcpListener::bind(format!("{host}:{port}")));
    server.run(app).await
}

pub fn main() {
    let result = start();

    if let Some(err) = result.err() {
        println!("Error: {err}");
    }
}
