use dto::dto;
use rocket::serde::json::Json;
use rocket_example_core::{Mutation, Query};

use sea_orm_rocket::Connection;

use rocket_okapi::okapi::openapi3::OpenApi;

use crate::error;
use crate::pool;
use pool::Db;

pub use entity::post;
pub use entity::post::Entity as Post;

use rocket_okapi::settings::OpenApiSettings;

use rocket_okapi::{openapi, openapi_get_routes_spec};

const DEFAULT_POSTS_PER_PAGE: u64 = 5;

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![settings: create, update, list, get_by_id, delete, destroy]
}

pub type R<T> = std::result::Result<rocket::serde::json::Json<T>, error::Error>;
pub type DataResult<'a, T> =
    std::result::Result<rocket::serde::json::Json<T>, rocket::serde::json::Error<'a>>;

/// # Add a new post
#[openapi(tag = "POST")]
#[post("/", data = "<post_data>")]
async fn create(
    conn: Connection<'_, Db>,
    post_data: DataResult<'_, post::Model>,
) -> R<Option<String>> {
    let db = conn.into_inner();
    let form = post_data?.into_inner();
    let cmd = Mutation::create_post(db, form);
    match cmd.await {
        Ok(_) => Ok(Json(Some("Post successfully added.".to_string()))),
        Err(e) => {
            let m = error::Error {
                err: "Could not insert post".to_string(),
                msg: Some(e.to_string()),
                http_status_code: 400,
            };
            Err(m)
        }
    }
}

/// # Update a post
#[openapi(tag = "POST")]
#[post("/<id>", data = "<post_data>")]
async fn update(
    conn: Connection<'_, Db>,
    id: i32,
    post_data: DataResult<'_, post::Model>,
) -> R<Option<String>> {
    let db = conn.into_inner();

    let form = post_data?.into_inner();

    let cmd = Mutation::update_post_by_id(db, id, form);
    match cmd.await {
        Ok(_) => Ok(Json(Some("Post successfully updated.".to_string()))),
        Err(e) => {
            let m = error::Error {
                err: "Could not update post".to_string(),
                msg: Some(e.to_string()),
                http_status_code: 400,
            };
            Err(m)
        }
    }
}

/// # Get post list
#[openapi(tag = "POST")]
#[get("/?<page>&<posts_per_page>")]
async fn list(
    conn: Connection<'_, Db>,
    page: Option<u64>,
    posts_per_page: Option<u64>,
) -> R<dto::PostsDto> {
    let db = conn.into_inner();

    // Set page number and items per page
    let page = page.unwrap_or(1);
    let posts_per_page = posts_per_page.unwrap_or(DEFAULT_POSTS_PER_PAGE);
    if page == 0 {
        let m = error::Error {
            err: "error getting posts".to_string(),
            msg: Some("'page' param cannot be zero".to_string()),
            http_status_code: 400,
        };
        return Err(m);
    }

    let (posts, num_pages) = Query::find_posts_in_page(db, page, posts_per_page)
        .await
        .expect("Cannot find posts in page");

    Ok(Json(dto::PostsDto {
        page,
        posts_per_page,
        num_pages,
        posts,
    }))
}

/// # get post by Id
#[openapi(tag = "POST")]
#[get("/<id>")]
async fn get_by_id(conn: Connection<'_, Db>, id: i32) -> R<Option<post::Model>> {
    let db = conn.into_inner();

    let post: Option<post::Model> = Query::find_post_by_id(db, id)
        .await
        .expect("could not find post");
    Ok(Json(post))
}

/// # delete post by Id
#[openapi(tag = "POST")]
#[delete("/<id>")]
async fn delete(conn: Connection<'_, Db>, id: i32) -> R<Option<String>> {
    let db = conn.into_inner();

    let cmd = Mutation::delete_post(db, id);
    match cmd.await {
        Ok(_) => Ok(Json(Some("Post successfully deleted.".to_string()))),
        Err(e) => {
            let m = error::Error {
                err: "Error deleting post".to_string(),
                msg: Some(e.to_string()),
                http_status_code: 400,
            };
            Err(m)
        }
    }
}

/// # delete all posts
#[openapi(tag = "POST")]
#[delete("/")]
async fn destroy(conn: Connection<'_, Db>) -> R<Option<String>> {
    let db = conn.into_inner();

    let cmd = Mutation::delete_all_posts(db);

    match cmd.await {
        Ok(_) => Ok(Json(Some(
            "All Posts were successfully deleted.".to_string(),
        ))),
        Err(e) => {
            let m = error::Error {
                err: "Error deleting all posts at once".to_string(),
                msg: Some(e.to_string()),
                http_status_code: 400,
            };
            Err(m)
        }
    }
}
