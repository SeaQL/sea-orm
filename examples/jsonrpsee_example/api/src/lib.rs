use std::env;

use anyhow::anyhow;
use entity::post;
use jsonrpsee::core::{async_trait, RpcResult};
use jsonrpsee::http_server::HttpServerBuilder;
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::types::error::CallError;
use jsonrpsee_example_core::sea_orm::{Database, DatabaseConnection};
use jsonrpsee_example_core::{Mutation, Query};
use log::info;
use migration::{Migrator, MigratorTrait};
use simplelog::*;
use std::fmt::Display;
use std::net::SocketAddr;
use tokio::signal::ctrl_c;
use tokio::signal::unix::{signal, SignalKind};

const DEFAULT_POSTS_PER_PAGE: u64 = 5;

#[rpc(server, client)]
trait PostRpc {
    #[method(name = "Post.List")]
    async fn list(
        &self,
        page: Option<u64>,
        posts_per_page: Option<u64>,
    ) -> RpcResult<Vec<post::Model>>;

    #[method(name = "Post.Insert")]
    async fn insert(&self, p: post::Model) -> RpcResult<i32>;

    #[method(name = "Post.Update")]
    async fn update(&self, p: post::Model) -> RpcResult<bool>;

    #[method(name = "Post.Delete")]
    async fn delete(&self, id: i32) -> RpcResult<bool>;
}

struct PpcImpl {
    conn: DatabaseConnection,
}

#[async_trait]
impl PostRpcServer for PpcImpl {
    async fn list(
        &self,
        page: Option<u64>,
        posts_per_page: Option<u64>,
    ) -> RpcResult<Vec<post::Model>> {
        let page = page.unwrap_or(1);
        let posts_per_page = posts_per_page.unwrap_or(DEFAULT_POSTS_PER_PAGE);

        Query::find_posts_in_page(&self.conn, page, posts_per_page)
            .await
            .map(|(p, _)| p)
            .internal_call_error()
    }

    async fn insert(&self, p: post::Model) -> RpcResult<i32> {
        let new_post = Mutation::create_post(&self.conn, p)
            .await
            .expect("could not insert post");

        Ok(new_post.id.unwrap())
    }

    async fn update(&self, p: post::Model) -> RpcResult<bool> {
        Mutation::update_post_by_id(&self.conn, p.id, p)
            .await
            .map(|_| true)
            .internal_call_error()
    }
    async fn delete(&self, id: i32) -> RpcResult<bool> {
        Mutation::delete_post(&self.conn, id)
            .await
            .map(|res| res.rows_affected == 1)
            .internal_call_error()
    }
}

trait IntoJsonRpcResult<T> {
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
async fn start() -> std::io::Result<()> {
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
    let conn = Database::connect(&db_url).await.unwrap();
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

pub fn main() {
    let result = start();

    if let Some(err) = result.err() {
        println!("Error: {}", err);
    }
}
