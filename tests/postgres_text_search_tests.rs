pub mod common {
    #[cfg(not(feature = "sync"))]
    pub mod runtime;
    pub mod setup;
    pub use setup::TestContext;
    pub mod features;
}

use sea_orm::{DbBackend, QueryTrait, entity::prelude::*};

mod article {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "article")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub title: String,
        pub body_tsv: TsVector,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

#[test]
fn test_tsvector_basic_traits() {
    let tsv = TsVector::from("sea & orm");
    assert_eq!(tsv.to_string(), "sea & orm");
    assert_eq!(&*tsv, "sea & orm");
    assert_eq!(tsv, TsVector("sea & orm".to_owned()));

    let tsq = TsQuery::from("sea | orm");
    assert_eq!(tsq.to_string(), "sea | orm");
    assert_eq!(&*tsq, "sea | orm");
    assert_eq!(tsq, TsQuery("sea | orm".to_owned()));
}

#[cfg(all(feature = "serde", feature = "with-json"))]
#[test]
fn test_tsvector_tsquery_serde() -> Result<(), serde_json::Error> {
    let tsv = TsVector::from("a:1 b:2");
    let json = serde_json::to_string(&tsv)?;
    assert_eq!(json, "\"a:1 b:2\"");
    let deserialized: TsVector = serde_json::from_str(&json)?;
    assert_eq!(deserialized, tsv);

    let tsq = TsQuery::from("a & b");
    let json_q = serde_json::to_string(&tsq)?;
    assert_eq!(json_q, "\"a & b\"");
    let deserialized_q: TsQuery = serde_json::from_str(&json_q)?;
    assert_eq!(deserialized_q, tsq);

    Ok(())
}

#[test]
fn test_matches_query_builder() {
    let stmt = article::Entity::find()
        .filter(article::COLUMN.body_tsv.matches(Expr::val("sea & orm")))
        .build(DbBackend::Postgres);

    assert_eq!(
        stmt.to_string(),
        r#"SELECT "article"."id", "article"."title", "article"."body_tsv" FROM "article" WHERE "article"."body_tsv" @@ 'sea & orm'"#
    );
}

