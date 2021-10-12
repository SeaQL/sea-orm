#[derive(Debug)]
pub enum DbErr {
    Conn(String),
    Exec(String),
    Query(String),
    RecordNotFound(String),
    Custom(String),
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
