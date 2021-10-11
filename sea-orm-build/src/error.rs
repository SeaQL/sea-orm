#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    SeaOrmCodegen(sea_orm_codegen::Error),
    Sqlx(sqlx::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}
