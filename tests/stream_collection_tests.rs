#![cfg(all(feature = "sqlx-sqlite", not(feature = "sync")))]

use std::sync::{Arc, Mutex};

use sea_orm::{
    Database, DatabaseExecutor, DbBackend, DbErr, FromQueryResult, QueryResult, SelectModel,
    SelectorRaw, Statement, TransactionTrait,
};
use tracing::{Instrument, Subscriber, field::Visit};
use tracing_subscriber::{Layer, layer::Context, prelude::*, registry::LookupSpan};

#[derive(Debug)]
struct ValueRow {
    value: i32,
}

impl FromQueryResult for ValueRow {
    fn from_query_result(row: &QueryResult, prefix: &str) -> Result<Self, DbErr> {
        assert_eq!(
            tracing::Span::current().metadata().unwrap().name(),
            "request"
        );
        Ok(Self {
            value: row.try_get(prefix, "value")?,
        })
    }
}

#[derive(Clone, Default)]
struct QuerySpans {
    completed: Arc<Mutex<Vec<CompletedSpan>>>,
}

#[derive(Default)]
struct QueryFields {
    status: Option<String>,
    statement: Option<String>,
}

impl Visit for QueryFields {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        match field.name() {
            "otel.status_code" => self.status = Some(value.to_owned()),
            "db.statement" => self.statement = Some(value.to_owned()),
            _ => {}
        }
    }

    fn record_debug(&mut self, _: &tracing::field::Field, _: &dyn std::fmt::Debug) {}
}

struct CompletedSpan {
    name: &'static str,
    #[cfg(feature = "tracing-spans")]
    fields: QueryFields,
}

impl<S> Layer<S> for QuerySpans
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(
        &self,
        attrs: &tracing::span::Attributes<'_>,
        id: &tracing::span::Id,
        ctx: Context<'_, S>,
    ) {
        let mut fields = QueryFields::default();
        attrs.record(&mut fields);
        ctx.span(id).unwrap().extensions_mut().insert(fields);
    }

    fn on_record(
        &self,
        id: &tracing::span::Id,
        values: &tracing::span::Record<'_>,
        ctx: Context<'_, S>,
    ) {
        values.record(
            ctx.span(id)
                .unwrap()
                .extensions_mut()
                .get_mut::<QueryFields>()
                .unwrap(),
        );
    }

    fn on_close(&self, id: tracing::span::Id, ctx: Context<'_, S>) {
        let span = ctx.span(&id).unwrap();
        self.completed.lock().unwrap().push(CompletedSpan {
            name: span.name(),
            #[cfg(feature = "tracing-spans")]
            fields: span.extensions_mut().remove::<QueryFields>().unwrap(),
        });
    }
}

#[tokio::test(flavor = "current_thread")]
async fn database_spans_cover_fetching_without_covering_model_decoding() {
    let mut db = Database::connect("sqlite::memory:").await.unwrap();
    let observed = Arc::new(Mutex::new(Vec::new()));
    let observed_in_callback = Arc::clone(&observed);
    db.set_metric_callback(move |_| {
        observed_in_callback
            .lock()
            .unwrap()
            .push(tracing::Span::current().metadata().unwrap().name());
    });

    let spans = QuerySpans::default();
    let _subscriber =
        tracing::subscriber::set_default(tracing_subscriber::registry().with(spans.clone()));
    async {
        let stmt = Statement::from_string(
            DbBackend::Sqlite,
            "SELECT 1 AS value UNION ALL SELECT 2 AS value",
        );
        let rows = SelectorRaw::<SelectModel<ValueRow>>::from_statement::<ValueRow>(stmt.clone())
            .all(&DatabaseExecutor::from(&db))
            .await
            .unwrap();
        assert_eq!(rows.iter().map(|row| row.value).collect::<Vec<_>>(), [1, 2]);

        let transaction = db.begin().await.unwrap();
        let rows = SelectorRaw::<SelectModel<ValueRow>>::from_statement::<ValueRow>(stmt)
            .all(&DatabaseExecutor::from(&transaction))
            .await
            .unwrap();
        assert_eq!(rows.iter().map(|row| row.value).collect::<Vec<_>>(), [1, 2]);
        transaction.rollback().await.unwrap();
    }
    .instrument(tracing::trace_span!("request"))
    .await;

    let observed = observed.lock().unwrap();
    assert_eq!(observed[0], "query_all");
    #[cfg(feature = "tracing-spans")]
    assert_eq!(observed[1], "sea_orm.query_all");
    #[cfg(not(feature = "tracing-spans"))]
    assert_eq!(observed[1], "query_all_raw");
    let completed = spans.completed.lock().unwrap();
    assert_eq!(
        completed
            .iter()
            .filter(|span| span.name == "query_all_raw")
            .count(),
        2
    );
    assert_eq!(
        completed
            .iter()
            .filter(|span| span.name == "query_all")
            .count(),
        1
    );
    #[cfg(feature = "tracing-spans")]
    {
        let database_spans = completed
            .iter()
            .filter(|span| span.name == "sea_orm.query_all")
            .collect::<Vec<_>>();
        assert_eq!(database_spans.len(), 2);
        for span in database_spans {
            assert_eq!(span.fields.status.as_deref(), Some("OK"));
            assert_eq!(
                span.fields.statement.as_deref(),
                Some("SELECT 1 AS value UNION ALL SELECT 2 AS value")
            );
        }
    }
}

#[cfg(feature = "tracing-spans")]
#[tokio::test(flavor = "current_thread")]
async fn model_decode_errors_do_not_mark_database_queries_as_failed() {
    async fn check_errors(executor: DatabaseExecutor<'_>) {
        for sql in [
            "SELECT 'invalid' AS value UNION ALL SELECT 2 AS value",
            "SELECT value FROM missing_table",
        ] {
            let error = SelectorRaw::<SelectModel<ValueRow>>::from_statement::<ValueRow>(
                Statement::from_string(DbBackend::Sqlite, sql),
            )
            .all(&executor)
            .await
            .unwrap_err();
            assert!(matches!(error, DbErr::Query(_)), "{error}");
        }
    }

    let db = Database::connect("sqlite::memory:").await.unwrap();
    let spans = QuerySpans::default();
    let _subscriber =
        tracing::subscriber::set_default(tracing_subscriber::registry().with(spans.clone()));
    async {
        check_errors(DatabaseExecutor::from(&db)).await;
        let transaction = db.begin().await.unwrap();
        check_errors(DatabaseExecutor::from(&transaction)).await;
        transaction.rollback().await.unwrap();
    }
    .instrument(tracing::trace_span!("request"))
    .await;
    let completed = spans.completed.lock().unwrap();
    let database_spans = completed
        .iter()
        .filter(|span| span.name == "sea_orm.query_all")
        .collect::<Vec<_>>();
    assert_eq!(database_spans.len(), 4);
    for span in database_spans {
        if span.fields.statement.as_deref() == Some("SELECT value FROM missing_table") {
            assert_eq!(span.fields.status.as_deref(), Some("ERROR"));
        } else {
            assert!(span.fields.status.is_none());
        }
    }
}
