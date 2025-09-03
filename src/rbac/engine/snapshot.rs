use super::*;

#[derive(Debug, Default)]
pub struct RbacSnapshot {
    pub(super) resources: Vec<Resource>,
    pub(super) permissions: Vec<Permission>,
    pub(super) roles: Vec<Role>,
    pub(super) user_roles: Vec<UserRole>,
    pub(super) role_permissions: Vec<RolePermission>,
    pub(super) user_overrides: Vec<UserOverride>,
    pub(super) role_hierarchy: Vec<RoleHierarchy>,
}

impl RbacSnapshot {
    /// Create an unrestricted system where `UserId(0)` can perform any action on any resource.
    /// This is intended to be an escape hatch to bypass RBAC restrictions.
    /// Use at your own risk.
    pub fn danger_unrestricted() -> Self {
        let mut snapshot = Self::default();

        snapshot.set_resources(vec![Resource {
            id: ResourceId(0),
            schema: None,
            table: WILDCARD.to_owned(),
        }]);
        snapshot.set_permissions(vec![Permission {
            id: PermissionId(0),
            action: WILDCARD.to_owned(),
        }]);
        snapshot.set_roles(vec![Role {
            id: RoleId(0),
            role: "unrestricted".to_owned(),
        }]);
        snapshot.set_user_role(UserId(0), "unrestricted");
        snapshot.add_role_permission("unrestricted", Action("*"), Table("*"));

        snapshot
    }

    pub(super) fn set_resources(&mut self, mut resources: Vec<Resource>) {
        for (i, r) in resources.iter_mut().enumerate() {
            r.id = ResourceId(i as i64 + 1);
        }
        self.resources = resources;
    }

    pub(super) fn set_permissions(&mut self, mut permissions: Vec<Permission>) {
        for (i, r) in permissions.iter_mut().enumerate() {
            r.id = PermissionId(i as i64 + 1);
        }
        self.permissions = permissions;
    }

    pub(super) fn set_roles(&mut self, mut roles: Vec<Role>) {
        for (i, r) in roles.iter_mut().enumerate() {
            r.id = RoleId(i as i64 + 1);
        }
        self.roles = roles;
    }

    pub(super) fn set_user_role(&mut self, user_id: UserId, role: &str) {
        self.user_roles.push(UserRole {
            user_id,
            role_id: self.find_role(role),
        });
    }

    pub(super) fn add_role_permission<P, R>(&mut self, role: &str, permission: P, resource: R)
    where
        P: Into<PermissionRequest>,
        R: Into<ResourceRequest>,
    {
        let permission = permission.into();
        let resource = resource.into();
        let role_id = self.find_role(role);
        self.role_permissions.push(RolePermission {
            role_id,
            permission_id: self.find_permission(&permission),
            resource_id: self.find_resource(&resource),
        });
    }

    #[cfg(test)]
    pub(super) fn add_user_override<P, R>(
        &mut self,
        user_id: UserId,
        permission: P,
        resource: R,
        grant: bool,
    ) where
        P: Into<PermissionRequest>,
        R: Into<ResourceRequest>,
    {
        let permission = permission.into();
        let resource = resource.into();
        self.user_overrides.push(UserOverride {
            user_id,
            permission_id: self.find_permission(&permission),
            resource_id: self.find_resource(&resource),
            grant,
        });
    }

    #[cfg(test)]
    pub(super) fn add_role_hierarchy(&mut self, role: &str, super_role: &str) {
        self.role_hierarchy.push(RoleHierarchy {
            role_id: self.find_role(role),
            super_role_id: self.find_role(super_role),
        })
    }

    pub(super) fn find_role(&self, role: &str) -> RoleId {
        self.roles.iter().find(|r| r.role == role).unwrap().id
    }

    pub(super) fn find_permission(&self, permission: &PermissionRequest) -> PermissionId {
        self.permissions
            .iter()
            .find(|r| r.action == permission.action)
            .unwrap()
            .id
    }

    pub(super) fn find_resource(&self, resource: &ResourceRequest) -> ResourceId {
        self.resources
            .iter()
            .find(|r| r.schema == resource.schema && r.table == resource.table)
            .unwrap()
            .id
    }
}
