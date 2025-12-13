use thiserror::Error;

/// An error from unsuccessful database operations
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Resource not found
    #[error("Resource Not Found: {0}")]
    ResourceNotFound(String),
    /// Permission not found
    #[error("Permission Not Found: {0}")]
    PermissionNotFound(String),
    /// Role not found
    #[error("Role Not Found: {0}")]
    RoleNotFound(String),
    /// User not found
    #[error("User Not Found: {0}")]
    UserNotFound(String),
}
