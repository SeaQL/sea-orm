/// An error from unsuccessful database operations
#[derive(Debug, PartialEq)]
pub enum DbErr {
    /// There was a problem with the database connection
    Conn(String),
    /// An operation did not execute successfully
    Exec(String),
    /// An error occurred while performing a query
    Query(String),
    /// The record was not found in the database
    RecordNotFound(String),
    /// A custom error
    Custom(String),
    /// Error occurred while parsing value as target type
    Type(String),
}

impl std::error::Error for DbErr {}

impl std::fmt::Display for DbErr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Conn(s) => write!(f, "Connection Error: {}", s),
            Self::Exec(s) => write!(f, "Execution Error: {}", s),
            Self::Query(s) => write!(f, "Query Error: {}", s),
            Self::RecordNotFound(s) => write!(f, "RecordNotFound Error: {}", s),
            Self::Custom(s) => write!(f, "Custom Error: {}", s),
            Self::Type(s) => write!(f, "Type Error: {}", s),
        }
    }
}

/// An error from a failed column operation when trying to convert the column to a string
#[derive(Debug, Clone)]
pub struct ColumnFromStrErr(pub String);

impl std::error::Error for ColumnFromStrErr {}

impl std::fmt::Display for ColumnFromStrErr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0.as_str())
    }
}
