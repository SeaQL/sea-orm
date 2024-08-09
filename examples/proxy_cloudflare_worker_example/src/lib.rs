use anyhow::Result;
use axum::{body::Body, response::Response};
use tower_service::Service;
use worker::{event, Context, Env, HttpRequest};

pub(crate) mod entity;
pub(crate) mod orm;
pub(crate) mod route;

// https://developers.cloudflare.com/workers/languages/rust
#[event(fetch)]
async fn fetch(req: HttpRequest, env: Env, _ctx: Context) -> Result<Response<Body>> {
    console_error_panic_hook::set_once();

    Ok(route::router(env).call(req).await?)
}
