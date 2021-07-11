pub mod schema;
use sea_orm::{Database, DatabaseConnection};
pub mod bakery_chain;
pub use bakery_chain::*;

macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}

pub struct TestContext {
    base_url: String,
    db_name: String,
    pub db_conn: DatabaseConnection,
}

impl TestContext {
    pub async fn new(base_url: &str, db_name: &str) -> Self {
        let db_conn = Database::connect("sqlite::memory:").await.unwrap();
        Self::setup_schema(&db_conn).await;

        Self {
            base_url: base_url.to_string(),
            db_name: db_name.to_string(),
            db_conn,
        }
    }

    async fn setup_schema(db: &DatabaseConnection) {
        assert!(schema::create_bakery_table(db).await.is_ok());
        assert!(schema::create_baker_table(db).await.is_ok());
        assert!(schema::create_customer_table(db).await.is_ok());
        assert!(schema::create_order_table(db).await.is_ok());
        assert!(schema::create_lineitem_table(db).await.is_ok());
        assert!(schema::create_cake_table(db).await.is_ok());
        assert!(schema::create_cakes_bakers_table(db).await.is_ok());
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        println!("dropping context");
    }
}
