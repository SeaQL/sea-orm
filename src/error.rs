/// Represents all the errors in SeaORM.
#[derive(Debug, PartialEq)]
pub enum DbErr {
    /// Error occurred while connecting to database engine.
    Conn(String),

    /// Error occurred while executing SQL statement.
    Exec(String),

    /// Error occurred while querying SQL statement.
    Query(String),

    /// Error occurred while updating a non-existing row in database.
    RecordNotFound(String),

    /// Error occurred while performing custom validation logics in [ActiveModelBehavior](crate::ActiveModelBehavior)
    Custom(String),

    /// Error occurred while parsing value into [ActiveEnum](crate::ActiveEnum)
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

#[derive(Debug, Clone)]
pub struct ColumnFromStrErr(pub String);

impl std::error::Error for ColumnFromStrErr {}

impl std::fmt::Display for ColumnFromStrErr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0.as_str())
    }
}
