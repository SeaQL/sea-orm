#![cfg(all(
    feature = "sqlx-sqlite",
    feature = "tracing-spans",
    not(feature = "sync")
))]

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use sea_orm::{
    Database, DbBackend, FromQueryResult, SelectModel, SelectorRaw, Statement, TransactionTrait,
};

#[derive(Debug, FromQueryResult)]
struct ValueRow {
    value: i32,
}

#[tokio::test(flavor = "current_thread")]
fn all_keeps_query_span_active_while_fetching_rows() {
    let mut db = Database::connect("sqlite::memory:").unwrap();
    let observed = Arc::new(AtomicBool::new(false));
    let observed_in_callback = Arc::clone(&observed);
    db.set_metric_callback(move |_| {
        if tracing::Span::current()
            .metadata()
            .is_some_and(|metadata| metadata.name() == "sea_orm.query_all")
        {
            observed_in_callback.store(true, Ordering::SeqCst);
        }
    });

    let _subscriber = tracing::subscriber::set_default(tracing_subscriber::registry());
    let stmt = Statement::from_string(
        DbBackend::Sqlite,
        "SELECT 1 AS value UNION ALL SELECT 2 AS value",
    );
    let rows = SelectorRaw::<SelectModel<ValueRow>>::from_statement::<ValueRow>(stmt.clone())
        .all(&db)
        .unwrap();

    assert_eq!(rows.iter().map(|row| row.value).collect::<Vec<_>>(), [1, 2]);
    assert!(observed.load(Ordering::SeqCst));

    observed.store(false, Ordering::SeqCst);
    let transaction = db.begin().unwrap();
    let rows = SelectorRaw::<SelectModel<ValueRow>>::from_statement::<ValueRow>(stmt)
        .all(&transaction)
        .unwrap();
    assert_eq!(rows.iter().map(|row| row.value).collect::<Vec<_>>(), [1, 2]);
    assert!(observed.load(Ordering::SeqCst));
    transaction.rollback().unwrap();
}
