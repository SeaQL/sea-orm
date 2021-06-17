#[macro_export]
#[cfg(feature = "debug-print")]
macro_rules! debug_print {
    ($( $args:expr ),*) => { println!( $( $args ),* ); }
}

#[macro_export]
// Non-debug version
#[cfg(not(feature = "debug-print"))]
macro_rules! debug_print {
    ($( $args:expr ),*) => {
        true;
    };
}

#[cfg(test)]
#[cfg(feature = "mock")]
pub(crate) fn get_mock_transaction_log(db: crate::Database) -> Vec<crate::Statement> {
    let mock_conn = match db.get_connection() {
        crate::DatabaseConnection::MockDatabaseConnection(mock_conn) => mock_conn,
        _ => unreachable!(),
    };
    let mut mocker = mock_conn.mocker.lock().unwrap();
    mocker.into_transaction_log()
}

#[macro_export]
#[cfg(test)]
#[cfg(feature = "mock")]
macro_rules! match_transaction_log {
    ($logs:expr, $stmts:expr, $query_builder:expr, $build_method:ident) => {
        for stmt in $stmts.iter() {
            assert!(!$logs.is_empty());
            let log = $logs.first().unwrap();
            let statement = $query_builder.$build_method(stmt);
            assert_eq!(log.to_string(), statement.to_string());
            $logs = $logs.drain(1..).collect();
        }
    };
}
