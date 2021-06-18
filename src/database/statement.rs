use sea_query::{inject_parameters, MySqlQueryBuilder, Values};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub sql: String,
    pub values: Option<Values>,
}

impl From<(String, Values)> for Statement {
    fn from(stmt: (String, Values)) -> Statement {
        Statement {
            sql: stmt.0,
            values: Some(stmt.1),
        }
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.values {
            Some(values) => {
                let string =
                    inject_parameters(&self.sql, values.0.clone(), &MySqlQueryBuilder::default());
                write!(f, "{}", &string)
            }
            None => {
                write!(f, "{}", &self.sql)
            }
        }
    }
}
