use std::collections::HashMap;

// use sea_orm::sea_query::tests_cfg::json;
use sea_orm::{ConnectOptions, Database, EntityTrait};

mod entity;
use entity::{underscores, underscores_workaround};

#[tokio::main]
async fn main() {
    let url = option_env!("DATABASE_URL");
    if let Some(url) = url {
        let opts = ConnectOptions::new(url.to_string());
        let conn = Database::connect(opts).await.unwrap();

        let results = underscores::Entity::find().all(&conn).await;
        dbg!(results);

        let results_workaround = underscores_workaround::Entity::find().all(&conn).await;
        dbg!(results_workaround);
    }

    let control = HashMap::from([
        ("a_b_c_d", 1i32),
        ("a_b_c_dd", 2i32),
        ("a_b_cc_d", 3i32),
        ("a_bb_c_d", 4i32),
        ("aa_b_c_d", 5i32),
    ]);
    // let control = json!(control);
    dbg!(control);
}
