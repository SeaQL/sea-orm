mod post;
mod setup;

use futures_util::StreamExt;
use post::Entity as Post;
use sea_orm::{prelude::*, Database};
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env::set_var("RUST_LOG", "debug");
    tracing_subscriber::fmt::init();

    dotenv::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let db = Database::connect(db_url)
        .await
        .expect("Database connection failed");
    let _ = setup::create_post_table(&db);
    tokio::task::spawn(async move {
        let mut stream = Post::find().stream(&db).await.unwrap();
        while let Some(item) = stream.next().await {
            let item = item?;
            println!("got something: {}", item.text);
        }
        Ok::<(), anyhow::Error>(())
    })
    .await?
}
