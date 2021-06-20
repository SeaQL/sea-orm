use crate::{ExecErr, TypeErr};

impl From<sqlx::Error> for TypeErr {
    fn from(_: sqlx::Error) -> TypeErr {
        TypeErr
    }
}

impl From<sqlx::Error> for ExecErr {
    fn from(_: sqlx::Error) -> ExecErr {
        ExecErr
    }
}