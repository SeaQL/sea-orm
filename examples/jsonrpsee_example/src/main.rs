use std::env;

use anyhow::anyhow;
use entity::post;
use entity::sea_orm;
use jsonrpsee::core::{async_trait, RpcResult};
use jsonrpsee::http_server::HttpServerBuilder;
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::types::error::CallError;
use log::info;
use migration::{Migrator, MigratorTrait};
use sea_orm::NotSet;
use sea_orm::{entity::*, query::*, DatabaseConnection};
use simplelog::*;
use std::fmt::Display;
use std::net::SocketAddr;
use tokio::signal::ctrl_c;
use tokio::signal::unix::{signal, SignalKind};

const DEFAULT_POSTS_PER_PAGE: usize = 5;

#[rpc(server, client)]
pub trait PostRpc {
    #[method(name = "Post.List")]
    async fn list(
        &self,
        page: Option<usize>,
        posts_per_page: Option<usize>,
    ) -> RpcResult<Vec<post::Model>>;

    #[method(name = "Post.Insert")]
    async fn insert(&self, p: post::Model) -> RpcResult<i32>;

    #[method(name = "Post.Update")]
    async fn update(&self, p: post::Model) -> RpcResult<bool>;

    #[method(name = "Post.Delete")]
    async fn delete(&self, id: i32) -> RpcResult<bool>;
}

pub struct PpcImpl {
    conn: DatabaseConnection,
}

#[async_trait]
impl PostRpcServer for PpcImpl {
    async fn list(
        &self,
        page: Option<usize>,
        posts_per_page: Option<usize>,
    ) -> RpcResult<Vec<post::Model>> {
        let page = page.unwrap_or(1);
        let posts_per_page = posts_per_page.unwrap_or(DEFAULT_POSTS_PER_PAGE);
        let paginator = post::Entity::find()
            .order_by_asc(post::Column::Id)
            .paginate(&self.conn, posts_per_page);
        paginator.fetch_page(page - 1).await.internal_call_error()
    }

    async fn insert(&self, p: post::Model) -> RpcResult<i32> {
        let active_post = post::ActiveModel {
            id: NotSet,
            title: Set(p.title),
            text: Set(p.text),
        };
        let new_post = active_post.insert(&self.conn).await.internal_call_error()?;
        Ok(new_post.id)
    }

    async fn update(&self, p: post::Model) -> RpcResult<bool> {
        let update_post = post::ActiveModel {
            id: Set(p.id),
            title: Set(p.title),
            text: Set(p.text),
        };
        update_post
            .update(&self.conn)
            .await
            .map(|_| true)
            .internal_call_error()
    }
    async fn delete(&self, id: i32) -> RpcResult<bool> {
        let post = post::Entity::find_by_id(id)
            .one(&self.conn)
            .await
            .internal_call_error()?;

        post.unwrap()
            .delete(&self.conn)
            .await
            .map(|res| res.rows_affected == 1)
            .internal_call_error()
    }
}

pub trait IntoJsonRpcResult<T> {
    fn internal_call_error(self) -> RpcResult<T>;
}

impl<T, E> IntoJsonRpcResult<T> for Result<T, E>
where
    E: Display,
{
    fn internal_call_error(self) -> RpcResult<T> {
        self.map_err(|e| jsonrpsee::core::Error::Call(CallError::Failed(anyhow!("{}", e))))
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let _ = TermLogger::init(
        LevelFilter::Trace,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    );

    // get env vars
    dotenv::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let server_url = format!("{}:{}", host, port);

    // create post table if not exists
    let conn = sea_orm::Database::connect(&db_url).await.unwrap();
    Migrator::up(&conn, None).await.unwrap();

    let server = HttpServerBuilder::default()
        .build(server_url.parse::<SocketAddr>().unwrap())
        .unwrap();

    let rpc_impl = PpcImpl { conn };
    let server_addr = server.local_addr().unwrap();
    let handle = server.start(rpc_impl.into_rpc()).unwrap();

    info!("starting listening {}", server_addr);
    let mut sig_int = signal(SignalKind::interrupt()).unwrap();
    let mut sig_term = signal(SignalKind::terminate()).unwrap();

    tokio::select! {
        _ = sig_int.recv() => info!("receive SIGINT"),
        _ = sig_term.recv() => info!("receive SIGTERM"),
        _ = ctrl_c() => info!("receive Ctrl C"),
    }
    handle.stop().unwrap();
    info!("Shutdown program");
    Ok(())
}
