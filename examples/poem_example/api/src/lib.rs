use std::env;
use std::sync::Arc;

use entity::post;
use migration::{Migrator, MigratorTrait};
use poem::endpoint::StaticFilesEndpoint;
use poem::error::InternalServerError;
use poem::http::StatusCode;
use poem::listener::TcpListener;
use poem::web::{Data, Form, Html, Path, Query};
use poem::{get, handler, post, EndpointExt, Error, IntoResponse, Result, Route, Server, Endpoint};
use poem_example_core::{
    sea_orm::{Database, DatabaseConnection, DatabaseTransaction, TransactionTrait},
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
async fn create(txn: Data<&Arc<DatabaseTransaction>>, form: Form<post::Model>) -> Result<impl IntoResponse> {
    let form = form.0;

    MutationCore::create_post(txn.0, form)
        .await
        .map_err(InternalServerError)?;

    Ok(StatusCode::FOUND.with_header("location", "/"))
}

#[handler]
async fn list(state: Data<&AppState>, txn: Data<&Arc<DatabaseTransaction>>, Query(params): Query<Params>) -> Result<impl IntoResponse> {
    let page = params.page.unwrap_or(1);
    let posts_per_page = params.posts_per_page.unwrap_or(DEFAULT_POSTS_PER_PAGE);

    let (posts, num_pages) = QueryCore::find_posts_in_page(txn.0, page, posts_per_page)
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
async fn edit(state: Data<&AppState>, txn: Data<&Arc<DatabaseTransaction>>, Path(id): Path<i32>) -> Result<impl IntoResponse> {

    let post: post::Model = QueryCore::find_post_by_id(txn.0, id)
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
    txn: Data<&Arc<DatabaseTransaction>>,
    Path(id): Path<i32>,
    form: Form<post::Model>,
) -> Result<impl IntoResponse> {
    let form = form.0;

    MutationCore::update_post_by_id(txn.0, id, form)
        .await
        .map_err(InternalServerError)?;

    Ok(StatusCode::FOUND.with_header("location", "/"))
}

#[handler]
async fn delete(txn: Data<&Arc<DatabaseTransaction>>, Path(id): Path<i32>) -> Result<impl IntoResponse> {
    MutationCore::delete_post(txn.0, id)
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
    let server_url = format!("{}:{}", host, port);

    // create post table if not exists
    let conn = Database::connect(&db_url).await.unwrap();
    Migrator::up(&conn, None).await.unwrap();
    let templates = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
    let state = AppState { templates, conn: conn.clone() };

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
        .around(|ep, mut req| async move {
            let db = &req
                .extensions()
                .get::<AppState>()
                .expect("DB not found in request data")
                .conn;
            let arc = match db.begin().await {
                Ok(v) => { Arc::new(v) }
                Err(err) => {
                    println!("{}", err);
                    return Ok(Error::from_status(StatusCode::INTERNAL_SERVER_ERROR).into_response());
                }
            };
            req.extensions_mut().insert(arc.clone());
            let resp = ep.get_response(req).await;

            let transaction = match Arc::try_unwrap(arc) {
                Ok(v) => {v}
                Err(_) => {
                    println!("Transaction arc has strong references!");
                    return Ok(Error::from_status(StatusCode::INTERNAL_SERVER_ERROR).into_response());}
            };
            if resp.status().is_success() || vec![StatusCode::FOUND, StatusCode::NOT_MODIFIED, StatusCode::SEE_OTHER].contains(&resp.status()) {
                if let Err(err) = transaction.commit().await {
                    println!("{}", err);
                    return Ok(Error::from_status(StatusCode::INTERNAL_SERVER_ERROR).into_response());
                };
            } else if let Err(err) = transaction.rollback().await {
                println!("{}", err);
                return Ok(Error::from_status(StatusCode::INTERNAL_SERVER_ERROR).into_response());
            }

            Ok(resp)
        })
        .data(state); // DO NOT MOVE THIS BEFORE THE `around` CALL!!!!!!
    let server = Server::new(TcpListener::bind(format!("{}:{}", host, port)));
    server.run(app).await
}

pub fn main() {
    let result = start();

    if let Some(err) = result.err() {
        println!("Error: {}", err);
    }
}
