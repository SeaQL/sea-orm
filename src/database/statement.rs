use sea_query::{inject_parameters, MySqlQueryBuilder, Values};
use std::{fmt};

pub struct Statement {
    pub sql: String,
    pub values: Values,
}

impl From<(String, Values)> for Statement {
    fn from(stmt: (String, Values)) -> Statement {
        Statement {
            sql: stmt.0,
            values: stmt.1,
        }
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = inject_parameters(
            &self.sql,
            self.values.0.clone(),
            &MySqlQueryBuilder::default(),
        );
        write!(f, "{}", &string)
    }
}
