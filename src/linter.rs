use crate::{AnyStatement, ConnectionTrait, StatementBuilderPlugin};

#[derive(Debug)]
pub struct QueryLinter<'a, C>
where
    C: ConnectionTrait,
{
    db: &'a C,
}

impl<'a, C> QueryLinter<'a, C>
where
    C: ConnectionTrait,
{
    pub fn new(db: &'a C) -> Self {
        Self { db }
    }
}

impl<C> StatementBuilderPlugin for QueryLinter<'_, C>
where
    C: ConnectionTrait,
{
    fn run(&self, stmt: &AnyStatement) {
        match stmt {
            AnyStatement::Insert(stmt) => InsertQueryLinter::run(stmt),
            AnyStatement::Select(stmt) => SelectQueryLinter::run(stmt),
            AnyStatement::Update(stmt) => UpdateQueryLinter::run(stmt),
            AnyStatement::Delete(stmt) => DeleteQueryLinter::run(stmt),
            _ => {}
        }
    }
}

#[derive(Debug)]
pub struct InsertQueryLinter;

impl InsertQueryLinter {
    fn run(stmt: &sea_query::InsertStatement) {
        dbg!(stmt);
        panic!("InsertQueryLinter invoked!");
    }
}

#[derive(Debug)]
pub struct SelectQueryLinter;

impl SelectQueryLinter {
    fn run(stmt: &sea_query::SelectStatement) {
        dbg!(stmt);
        panic!("SelectQueryLinter invoked!");
    }
}

#[derive(Debug)]
pub struct UpdateQueryLinter;

impl UpdateQueryLinter {
    fn run(stmt: &sea_query::UpdateStatement) {
        dbg!(stmt);
        panic!("UpdateQueryLinter invoked!");
    }
}

#[derive(Debug)]
pub struct DeleteQueryLinter;

impl DeleteQueryLinter {
    fn run(stmt: &sea_query::DeleteStatement) {
        dbg!(stmt);
        panic!("DeleteQueryLinter invoked!");
    }
}
