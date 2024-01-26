use loco_rs::prelude::*;

use crate::{models::_entities::users, views::user::CurrentResponse};

async fn current(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Json<CurrentResponse>> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    format::json(CurrentResponse::new(&user))
}

pub fn routes() -> Routes {
    Routes::new().prefix("user").add("/current", get(current))
}
