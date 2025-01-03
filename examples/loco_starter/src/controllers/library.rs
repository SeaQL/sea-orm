use loco_rs::prelude::*;

pub async fn config(State(_ctx): State<AppContext>) -> Result<Response> {
    format::json(serde_json::json!({
        "site": {
            "theme": {
                "title": "SeaORM Pro FREE",
                "logo": "/admin/favicon.ico",
                "login_banner": "/admin/logo.png",
                "copyright": "Powered by SeaORM Pro",
            }
        },
        "raw_tables": {},
        "composite_tables": {},
    }))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("library")
        .add("/config", get(config))
}
