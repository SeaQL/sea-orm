mod flash;
mod graphql;

use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::{Form, Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Json},
    routing::{get, get_service, post},
    Router,
};
use axum_example_service::{
    sea_orm::{Database, DatabaseConnection},
    Mutation as MutationCore, Query as QueryCore,
};
use entity::post;
use flash::{get_flash_cookie, post_response, PostResponse};
use migration::{Migrator, MigratorTrait};
use serde::{Deserialize, Serialize};
use std::env;
use tera::Tera;
use tower_cookies::{CookieManagerLayer, Cookies};
use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn start() -> anyhow::Result<()> {
    env::set_var("RUST_LOG", "debug");
    tracing_subscriber::fmt::init();

    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let server_url = format!("{host}:{port}");

    let conn = Database::connect(db_url)
        .await
        .expect("Database connection failed");
    Migrator::up(&conn, None).await.unwrap();

    let templates = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*"))
        .expect("Tera initialization failed");

    let state = AppState { templates, conn };

    let app = Router::new()
        .route("/", get(list_posts).post(create_post))
        .route("/:id", get(edit_post).post(update_post))
        .route("/new", get(new_post))
        .route("/delete/:id", post(delete_post))
        .route("/api/admin/config", get(admin_panel_config))
        .route("/api/auth/login", post(user_login))
        .route("/api/user/current", get(current_user))
        .route("/api/graphql", get(graphql_playground))
        .route("/api/graphql", post(graphql_handler))
        .nest_service(
            "/static",
            get_service(ServeDir::new(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/static"
            )))
            .handle_error(|error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {error}"),
                )
            }),
        )
        .nest_service(
            "/admin",
            get_service(
                ServeDir::new(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/admin")).fallback(
                    ServeFile::new(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/assets/admin/index.html"
                    )),
                ),
            ),
        )
        .layer(CookieManagerLayer::new())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&server_url).await.unwrap();
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
struct AppState {
    templates: Tera,
    conn: DatabaseConnection,
}

#[derive(Deserialize)]
struct Params {
    page: Option<u64>,
    posts_per_page: Option<u64>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct FlashData {
    kind: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PasswordLoginParams {
    pub email: String,
    pub password: String,
}

async fn list_posts(
    state: State<AppState>,
    Query(params): Query<Params>,
    cookies: Cookies,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let page = params.page.unwrap_or(1);
    let posts_per_page = params.posts_per_page.unwrap_or(5);

    let (posts, num_pages) = QueryCore::find_posts_in_page(&state.conn, page, posts_per_page)
        .await
        .expect("Cannot find posts in page");

    let mut ctx = tera::Context::new();
    ctx.insert("posts", &posts);
    ctx.insert("page", &page);
    ctx.insert("posts_per_page", &posts_per_page);
    ctx.insert("num_pages", &num_pages);

    if let Some(value) = get_flash_cookie::<FlashData>(&cookies) {
        ctx.insert("flash", &value);
    }

    let body = state
        .templates
        .render("index.html.tera", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

    Ok(Html(body))
}

async fn new_post(state: State<AppState>) -> Result<Html<String>, (StatusCode, &'static str)> {
    let ctx = tera::Context::new();
    let body = state
        .templates
        .render("new.html.tera", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

    Ok(Html(body))
}

async fn create_post(
    state: State<AppState>,
    mut cookies: Cookies,
    form: Form<post::Model>,
) -> Result<PostResponse, (StatusCode, &'static str)> {
    let form = form.0;

    MutationCore::create_post(&state.conn, form)
        .await
        .expect("could not insert post");

    let data = FlashData {
        kind: "success".to_owned(),
        message: "Post successfully added".to_owned(),
    };

    Ok(post_response(&mut cookies, data))
}

async fn edit_post(
    state: State<AppState>,
    Path(id): Path<i32>,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let post: post::Model = QueryCore::find_post_by_id(&state.conn, id)
        .await
        .expect("could not find post")
        .unwrap_or_else(|| panic!("could not find post with id {id}"));

    let mut ctx = tera::Context::new();
    ctx.insert("post", &post);

    let body = state
        .templates
        .render("edit.html.tera", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

    Ok(Html(body))
}

async fn update_post(
    state: State<AppState>,
    Path(id): Path<i32>,
    mut cookies: Cookies,
    form: Form<post::Model>,
) -> Result<PostResponse, (StatusCode, String)> {
    let form = form.0;

    MutationCore::update_post_by_id(&state.conn, id, form)
        .await
        .expect("could not edit post");

    let data = FlashData {
        kind: "success".to_owned(),
        message: "Post successfully updated".to_owned(),
    };

    Ok(post_response(&mut cookies, data))
}

async fn delete_post(
    state: State<AppState>,
    Path(id): Path<i32>,
    mut cookies: Cookies,
) -> Result<PostResponse, (StatusCode, &'static str)> {
    MutationCore::delete_post(&state.conn, id)
        .await
        .expect("could not delete post");

    let data = FlashData {
        kind: "success".to_owned(),
        message: "Post successfully deleted".to_owned(),
    };

    Ok(post_response(&mut cookies, data))
}

const DEMO_USER: &str = "demo@sea-ql.org";
const DEMO_USER_PID: &str = "79a6243b-088d-5d95-9b16-a2d1689e291f";
const DEMO_USER_TOKEN: &str = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzUxMiJ9.eyJwaWQiOiI2MjRhOWMxZi1hMTQ5LTQ0Y2MtYjBhMy03OTMzNDViZTlkOTMiLCJleHAiOjE3MzY4NDc1OTcsImNsYWltcyI6bnVsbH0.w2dJzWUw343eAt_sWrngb065uwJK-SOgJ8gDBls7XHSKILNKGzh-ZG9VFEBwVl4356-vD1MM8Qo8Y2TcO5V-NA";

async fn admin_panel_config() -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
    Ok(Json(serde_json::json!({
        "site": {
            "theme": {
                "title": "SeaORM Pro",
                "logo": "/admin/favicon.ico",
                "login_banner": "/admin/logo.png",
            }
        },
        "raw_tables": {},
        "composite_tables": {},
    })))
}

async fn user_login(
    params: Json<PasswordLoginParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
    if params.email != DEMO_USER {
        panic!("unauthorized!");
    }
    if params.password != DEMO_USER {
        panic!("unauthorized!");
    }

    Ok(Json(serde_json::json!({
        "token": DEMO_USER_TOKEN,
        "pid": DEMO_USER_PID,
        "name": "Demo User",
        "is_verified": true,
    })))
}

fn check_user_auth(headers: &HeaderMap) -> Result<(), (StatusCode, &'static str)> {
    let auth_header = headers.get("AUTHORIZATION");

    let Some(auth_header) = auth_header else {
        panic!("unauthorized!");
    };
    if !auth_header.to_str().unwrap().ends_with(DEMO_USER_TOKEN) {
        panic!("unauthorized!");
    }

    Ok(())
}

async fn current_user(
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
    check_user_auth(&headers)?;

    Ok(Json(serde_json::json!({
        "pid": DEMO_USER_PID,
        "name": "Demo User",
        "email": DEMO_USER,
    })))
}

async fn graphql_playground() -> impl IntoResponse {
    // Setup GraphQL playground web and specify the endpoint for GraphQL resolver
    let res = playground_source(GraphQLPlaygroundConfig::new("/api/graphql"));

    Html(res)
}

async fn graphql_handler(
    state: State<AppState>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> Result<GraphQLResponse, (StatusCode, &'static str)> {
    check_user_auth(&headers)?;
    // Maximum depth of the constructed query
    const DEPTH: Option<usize> = None;
    // Maximum complexity of the constructed query
    const COMPLEXITY: Option<usize> = None;
    // GraphQL schema
    let schema = graphql::query_root::schema(state.conn.clone(), DEPTH, COMPLEXITY).unwrap();
    // GraphQL handler
    let res = schema.execute(req.into_inner()).await.into();
    Ok(res)
}

pub fn main() {
    let result = start();

    if let Some(err) = result.err() {
        println!("Error: {err}");
    }
}
