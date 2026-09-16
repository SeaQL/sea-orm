use super::*;
use futures_util::StreamExt;
use std::sync::{Arc, Mutex};

#[smol_potat::test]
async fn reports_stream_errors() {
    let observed = Arc::new(Mutex::new(None));
    let metric_callback = Some({
        let observed = Arc::clone(&observed);
        Arc::new(move |info: &crate::metric::Info<'_>| {
            *observed.lock().expect("metric lock") = Some(info.failed);
        }) as crate::metric::Callback
    });
    let stmt = Statement::from_string(crate::DbBackend::Postgres, "SELECT 1");

    {
        let inner = futures_util::stream::once(async {
            Err(DbErr::Custom("expected query error".to_owned()))
        });
        let mut stream = MetricStream::new(&metric_callback, &stmt, Some(Duration::ZERO), inner);
        assert!(stream.next().await.expect("one item").is_err());
    }

    assert_eq!(*observed.lock().expect("metric lock"), Some(true));
}

#[smol_potat::test]
async fn includes_time_waiting_for_stream() {
    let observed = Arc::new(Mutex::new(None));
    let metric_callback = Some({
        let observed = Arc::clone(&observed);
        Arc::new(move |info: &crate::metric::Info<'_>| {
            *observed.lock().expect("metric lock") = Some(info.elapsed);
        }) as crate::metric::Callback
    });
    let stmt = Statement::from_string(crate::DbBackend::Postgres, "SELECT 1");
    let delay = Duration::from_millis(20);

    {
        let inner = futures_util::stream::once(async move {
            smol::Timer::after(delay).await;
            Err(DbErr::Custom("expected query error".to_owned()))
        });
        let mut stream = MetricStream::new(&metric_callback, &stmt, Some(Duration::ZERO), inner);
        assert!(stream.next().await.expect("one item").is_err());
    }

    assert!(observed.lock().expect("metric lock").expect("metric") >= delay);
}
