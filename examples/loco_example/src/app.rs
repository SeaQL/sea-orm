use std::path::Path;

use async_trait::async_trait;
use loco_rs::{
    app::{AppContext, Hooks},
    controller::AppRoutes,
    db::{self, truncate_table},
    task::Tasks,
    worker::Processor,
    Result,
};
use sea_orm::DatabaseConnection;

use crate::{controllers, models::_entities::notes};

pub struct App;
#[async_trait]
impl Hooks for App {
    fn app_name() -> &'static str {
        env!("CARGO_CRATE_NAME")
    }

    fn routes() -> AppRoutes {
        AppRoutes::with_default_routes()
            .prefix("/api")
            .add_route(controllers::notes::routes())
    }

    fn connect_workers<'a>(_p: &'a mut Processor, _ctx: &'a AppContext) {}

    fn register_tasks(_tasks: &mut Tasks) {}

    async fn truncate(db: &DatabaseConnection) -> Result<()> {
        truncate_table(db, notes::Entity).await?;
        Ok(())
    }

    async fn seed(db: &DatabaseConnection, base: &Path) -> Result<()> {
        db::seed::<notes::ActiveModel>(db, &base.join("notes.yaml").display().to_string()).await?;
        Ok(())
    }
}
