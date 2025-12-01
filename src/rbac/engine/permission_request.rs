use crate::rbac::entity::permission::Model as Permission;

#[derive(Debug)]
pub struct Action<'a>(pub &'a str);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PermissionRequest {
    pub action: String,
}

impl<'a> From<Action<'a>> for PermissionRequest {
    fn from(action: Action<'a>) -> PermissionRequest {
        PermissionRequest {
            action: action.0.to_owned(),
        }
    }
}

impl From<Permission> for PermissionRequest {
    fn from(permission: Permission) -> Self {
        Self {
            action: permission.action,
        }
    }
}
