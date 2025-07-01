use crate::entity::{
    permission::{Model as Permission, PermissionId},
    resource::{Model as Resource, ResourceId},
    role::{Model as Role, RoleId},
    role_hierarchy::Model as RoleHierarchy,
    role_permission::Model as RolePermission,
    user_override::Model as UserOverride,
    user_role::Model as UserRole,
};
use crate::{Error, WILDCARD};

mod permission_request;
mod resource_request;
mod role_hierarchy_impl;

pub use permission_request::*;
pub use resource_request::*;
use role_hierarchy_impl::*;

use std::collections::{HashMap, HashSet};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct UserId(i64);

pub struct RbacEngine {
    resources: HashMap<ResourceRequest, Resource>,
    permissions: HashMap<PermissionRequest, Permission>,
    wildcard_resources: HashMap<ResourceId, Resource>,
    wildcard_permissions: HashMap<PermissionId, Permission>,
    roles: HashSet<RoleId>,
    user_roles: HashMap<UserId, Vec<RoleId>>,
    role_permissions: HashMap<RoleId, HashSet<(PermissionId, ResourceId)>>,
    user_overrides: HashMap<UserId, Vec<UserOverride>>,
    role_hierarchy: HashMap<RoleId, Vec<RoleId>>, // Role -> ChildRole
}

impl RbacEngine {
    pub fn from_query_result(
        resources_rows: Vec<Resource>,
        permissions_rows: Vec<Permission>,
        roles_rows: Vec<Role>,
        user_roles_rows: Vec<UserRole>,
        role_permissions_rows: Vec<RolePermission>,
        user_overrides_rows: Vec<UserOverride>,
        role_hierarchy_rows: Vec<RoleHierarchy>,
    ) -> Self {
        // let mut resources = HashMap::new();
        let mut wildcard_resources = HashMap::new();
        for resource in resources_rows {
            if resource.schema.as_deref() == Some(WILDCARD) || resource.table == WILDCARD {
                wildcard_resources.insert(resource.id, resource);
            }
        }
        todo!()
    }

    pub fn user_can<P, R>(&self, user_id: UserId, permission: P, resource: R) -> Result<bool, Error>
    where
        P: Into<PermissionRequest>,
        R: Into<ResourceRequest>,
    {
        let resource = resource.into();
        let permission = permission.into();
        let resource = self
            .resources
            .get(&resource)
            .ok_or_else(|| Error::ResourceNotFound(format!("{resource:?}")))?;
        let permission = self
            .permissions
            .get(&permission)
            .ok_or_else(|| Error::PermissionNotFound(format!("{permission:?}")))?;

        // get user roles and flatten hierarchy
        let mut user_roles = HashSet::new();
        if let Some(roles) = self.user_roles.get(&user_id) {
            for role in roles {
                for role in enumerate_role(*role, &self.role_hierarchy) {
                    if !self.roles.contains(&role) {
                        return Err(Error::RoleNotFound(format!("{role:?}")));
                    }
                    user_roles.insert(role);
                }
            }
        }

        if let Some(user_overrides) = self.user_overrides.get(&user_id) {
            for user_override in user_overrides {
                if user_override.permission_id == permission.id
                    && user_override.resource_id == resource.id
                {
                    return Ok(user_override.grant);
                }
            }
        }

        for role_id in user_roles {
            if let Some(role_permissions) = self.role_permissions.get(&role_id) {
                if role_permissions.contains(&(permission.id, resource.id)) {
                    return Ok(true);
                }
                for (permission_id, resource_id) in role_permissions {
                    let is_wildcard_permission =
                        self.is_wildcard_permission(*permission_id, &permission);
                    let is_wildcard_resource = self.is_wildcard_resource(*resource_id, &resource);
                    if resource_id == &resource.id && is_wildcard_permission {
                        return Ok(true);
                    }
                    if permission_id == &permission.id && is_wildcard_resource {
                        return Ok(true);
                    }
                    if is_wildcard_permission && is_wildcard_resource {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    fn is_wildcard_resource(&self, id: ResourceId, target: &Resource) -> bool {
        if let Some(resource) = self.wildcard_resources.get(&id) {
            let schema_match = resource.schema.is_none()
                || resource.schema.as_ref().unwrap() == WILDCARD
                || resource.schema == target.schema;
            let table_match = resource.table == WILDCARD || resource.table == target.table;
            return schema_match && table_match;
        }
        false
    }

    fn is_wildcard_permission(&self, id: PermissionId, _: &Permission) -> bool {
        if let Some(permission) = self.wildcard_permissions.get(&id) {
            return permission.action == WILDCARD;
        }
        false
    }
}
