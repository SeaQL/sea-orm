use crate::{Database, IntoMockRow, MockDatabaseConnection, MockRow, QueryBuilderBackend, Statement, tests_cfg::*};
use sea_query::{SelectStatement, Value};

impl From<cake_filling::Model> for MockRow {
    fn from(model: cake_filling::Model) -> Self {
        let map = maplit::btreemap! {
            "cake_id" => Into::<Value>::into(model.cake_id),
            "filling_id" => Into::<Value>::into(model.filling_id),
        };
        map.into_mock_row()
    }
}

impl From<cake::Model> for MockRow {
    fn from(model: cake::Model) -> Self {
        let map = maplit::btreemap! {
            "id" => Into::<Value>::into(model.id),
            "name" => Into::<Value>::into(model.name),
        };
        map.into_mock_row()
    }
}

impl From<filling::Model> for MockRow {
    fn from(model: filling::Model) -> Self {
        let map = maplit::btreemap! {
            "id" => Into::<Value>::into(model.id),
            "name" => Into::<Value>::into(model.name),
        };
        map.into_mock_row()
    }
}

impl From<fruit::Model> for MockRow {
    fn from(model: fruit::Model) -> Self {
        let map = maplit::btreemap! {
            "id" => Into::<Value>::into(model.id),
            "name" => Into::<Value>::into(model.name),
            "cake_id" => Into::<Value>::into(model.cake_id),
        };
        map.into_mock_row()
    }
}

pub fn get_mock_db_connection(db: &Database) -> &MockDatabaseConnection {
    match db.get_connection() {
        crate::DatabaseConnection::MockDatabaseConnection(mock_conn) => mock_conn,
        _ => unreachable!(),
    }
}

pub fn get_mock_transaction_log(db: Database) -> Vec<Statement> {
    let mock_conn = get_mock_db_connection(&db);
    let mut mocker = mock_conn.mocker.lock().unwrap();
    mocker.into_transaction_log()
}

pub fn match_transaction_log(mut logs: Vec<Statement>, stmts: Vec<SelectStatement>, query_builder: &QueryBuilderBackend) -> Vec<Statement> {
    for stmt in stmts.iter() {
        assert!(!logs.is_empty());
        let log = logs.first().unwrap();
        let statement = query_builder.build_select_statement(stmt);
        assert_eq!(log.to_string(), statement.to_string());
        logs = logs.drain(1..).collect();
    }
    logs
}
