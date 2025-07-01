use thiserror::Error;

/// An error from unsuccessful database operations
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("Resource Not Found: {0}")]
    ResourceNotFound(String),
    #[error("Permission Not Found: {0}")]
    PermissionNotFound(String),
    #[error("Role Not Found: {0}")]
    RoleNotFound(String),
}
