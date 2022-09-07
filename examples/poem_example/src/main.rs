use std::env;

use entity::post;
use migration::{Migrator, MigratorTrait};
use poem::endpoint::StaticFilesEndpoint;
use poem::error::{BadRequest, InternalServerError};
use poem::http::StatusCode;
use poem::listener::TcpListener;
use poem::web::{Data, Form, Html, Path, Query};
use poem::{get, handler, post, EndpointExt, Error, IntoResponse, Result, Route, Server};
use sea_orm::{entity::*, query::*, DatabaseConnection};
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
    post::ActiveModel {
        title: Set(form.title.to_owned()),
        text: Set(form.text.to_owned()),
        ..Default::default()
    }
    .save(&state.conn)
    .await
    .map_err(InternalServerError)?;

    Ok(StatusCode::FOUND.with_header("location", "/"))
}

#[handler]
async fn list(state: Data<&AppState>, Query(params): Query<Params>) -> Result<impl IntoResponse> {
    let page = params.page.unwrap_or(1);
    let posts_per_page = params.posts_per_page.unwrap_or(DEFAULT_POSTS_PER_PAGE);
    let paginator = post::Entity::find()
        .order_by_asc(post::Column::Id)
        .paginate(&state.conn, posts_per_page);
    let num_pages = paginator.num_pages().await.map_err(BadRequest)?;
    let posts = paginator
        .fetch_page(page - 1)
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
    let post: post::Model = post::Entity::find_by_id(id)
        .one(&state.conn)
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
    post::ActiveModel {
        id: Set(id),
        title: Set(form.title.to_owned()),
        text: Set(form.text.to_owned()),
    }
    .save(&state.conn)
    .await
    .map_err(InternalServerError)?;

    Ok(StatusCode::FOUND.with_header("location", "/"))
}

#[handler]
async fn delete(state: Data<&AppState>, Path(id): Path<i32>) -> Result<impl IntoResponse> {
    let post: post::ActiveModel = post::Entity::find_by_id(id)
        .one(&state.conn)
        .await
        .map_err(InternalServerError)?
        .ok_or_else(|| Error::from_status(StatusCode::NOT_FOUND))?
        .into();
    post.delete(&state.conn)
        .await
        .map_err(InternalServerError)?;

    Ok(StatusCode::FOUND.with_header("location", "/"))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    tracing_subscriber::fmt::init();

    // get env vars
    dotenv::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let server_url = format!("{}:{}", host, port);

    // create post table if not exists
    let conn = sea_orm::Database::connect(&db_url).await.unwrap();
    Migrator::up(&conn, None).await.unwrap();
    let templates = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
    let state = AppState { templates, conn };

    println!("Starting server at {}", server_url);

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
    let server = Server::new(TcpListener::bind(format!("{}:{}", host, port)));
    server.run(app).await
}
