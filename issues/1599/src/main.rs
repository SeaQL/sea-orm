use anyhow::Result;

pub mod cake;
pub mod cake_filling;
pub mod filling;
pub mod fruit;

#[tokio::main]
async fn main() -> Result<()> {
    Ok(())
}
