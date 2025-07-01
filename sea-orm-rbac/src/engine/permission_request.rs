use crate::entity::resource::Model as Resource;

pub struct Action<'a>(&'a str);

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
