use crate::QueryBuilderWithSyntax;
use sea_query::{
    inject_parameters, MysqlQueryBuilder, PostgresQueryBuilder, QueryBuilder, SqliteQueryBuilder,
    Values,
};
use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Syntax {
    MySql,
    Postgres,
    Sqlite,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub sql: String,
    pub values: Option<Values>,
    pub syntax: Syntax,
}

impl Statement {
    pub fn from_string(syntax: Syntax, stmt: String) -> Statement {
        Statement {
            sql: stmt,
            values: None,
            syntax,
        }
    }

    pub fn from_string_values_tuple(syntax: Syntax, stmt: (String, Values)) -> Statement {
        Statement {
            sql: stmt.0,
            values: Some(stmt.1),
            syntax,
        }
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.values {
            Some(values) => {
                let string = inject_parameters(
                    &self.sql,
                    values.0.clone(),
                    self.syntax.get_query_builder().as_ref(),
                );
                write!(f, "{}", &string)
            }
            None => {
                write!(f, "{}", &self.sql)
            }
        }
    }
}

impl Syntax {
    pub fn get_query_builder(&self) -> Box<dyn QueryBuilder> {
        match self {
            Self::MySql => Box::new(MysqlQueryBuilder),
            Self::Postgres => Box::new(PostgresQueryBuilder),
            Self::Sqlite => Box::new(SqliteQueryBuilder),
        }
    }
}

impl QueryBuilderWithSyntax for MysqlQueryBuilder {
    fn syntax(&self) -> Syntax {
        Syntax::MySql
    }
}

impl QueryBuilderWithSyntax for PostgresQueryBuilder {
    fn syntax(&self) -> Syntax {
        Syntax::Postgres
    }
}

impl QueryBuilderWithSyntax for SqliteQueryBuilder {
    fn syntax(&self) -> Syntax {
        Syntax::Sqlite
    }
}
