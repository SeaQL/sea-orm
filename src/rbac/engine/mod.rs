use super::entity::{
    permission::{Model as Permission, PermissionId},
    resource::{Model as Resource, ResourceId},
    role::{Model as Role, RoleId},
    role_hierarchy::Model as RoleHierarchy,
    role_permission::Model as RolePermission,
    user::UserId,
    user_override::Model as UserOverride,
    user_role::Model as UserRole,
};
use super::{Error, WILDCARD};

mod loader;
mod permission_request;
mod resource_request;
mod role_hierarchy_impl;

pub use permission_request::*;
pub use resource_request::*;
use role_hierarchy_impl::*;

use std::collections::{HashMap, HashSet};

pub struct RbacEngine {
    resources: HashMap<ResourceRequest, Resource>,
    permissions: HashMap<PermissionRequest, Permission>,
    wildcard_resources: HashMap<ResourceId, Resource>,
    wildcard_permissions: HashMap<PermissionId, Permission>,
    roles: HashMap<RoleId, Role>,
    user_roles: HashMap<UserId, RoleId>,
    role_permissions: HashMap<RoleId, HashSet<(PermissionId, ResourceId)>>,
    user_overrides: HashMap<UserId, Vec<UserOverride>>,
    role_hierarchy: HashMap<RoleId, Vec<RoleId>>, // Role -> ChildRole
}

impl std::fmt::Debug for RbacEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RbacEngine")
    }
}

#[derive(Debug, Default)]
pub struct RbacSnapshot {
    resources: Vec<Resource>,
    permissions: Vec<Permission>,
    roles: Vec<Role>,
    user_roles: Vec<UserRole>,
    role_permissions: Vec<RolePermission>,
    user_overrides: Vec<UserOverride>,
    role_hierarchy: Vec<RoleHierarchy>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RbacUserRolePermissions {
    pub role: Role,
    pub permissions: Vec<(Resource, Permission)>,
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

    fn set_resources(&mut self, mut resources: Vec<Resource>) {
        for (i, r) in resources.iter_mut().enumerate() {
            r.id = ResourceId(i as i64 + 1);
        }
        self.resources = resources;
    }

    fn set_permissions(&mut self, mut permissions: Vec<Permission>) {
        for (i, r) in permissions.iter_mut().enumerate() {
            r.id = PermissionId(i as i64 + 1);
        }
        self.permissions = permissions;
    }

    fn set_roles(&mut self, mut roles: Vec<Role>) {
        for (i, r) in roles.iter_mut().enumerate() {
            r.id = RoleId(i as i64 + 1);
        }
        self.roles = roles;
    }

    fn set_user_role(&mut self, user_id: UserId, role: &str) {
        self.user_roles.push(UserRole {
            user_id,
            role_id: self.find_role(role),
        });
    }

    fn add_role_permission<P, R>(&mut self, role: &str, permission: P, resource: R)
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
    fn add_user_override<P, R>(&mut self, user_id: UserId, permission: P, resource: R, grant: bool)
    where
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
    fn add_role_hierarchy(&mut self, role: &str, super_role: &str) {
        self.role_hierarchy.push(RoleHierarchy {
            role_id: self.find_role(role),
            super_role_id: self.find_role(super_role),
        })
    }

    fn find_role(&self, role: &str) -> RoleId {
        self.roles.iter().find(|r| r.role == role).unwrap().id
    }

    fn find_permission(&self, permission: &PermissionRequest) -> PermissionId {
        self.permissions
            .iter()
            .find(|r| r.action == permission.action)
            .unwrap()
            .id
    }

    fn find_resource(&self, resource: &ResourceRequest) -> ResourceId {
        self.resources
            .iter()
            .find(|r| r.schema == resource.schema && r.table == resource.table)
            .unwrap()
            .id
    }
}

impl RbacEngine {
    pub fn from_snapshot(
        RbacSnapshot {
            resources: resources_rows,
            permissions: permissions_rows,
            roles: roles_rows,
            user_roles: user_roles_rows,
            role_permissions: role_permissions_rows,
            user_overrides: user_overrides_rows,
            role_hierarchy: role_hierarchy_rows,
        }: RbacSnapshot,
    ) -> Self {
        let mut resources: HashMap<ResourceRequest, Resource> = Default::default();
        let mut wildcard_resources = HashMap::new();
        for resource in resources_rows {
            if resource.schema.as_deref() == Some(WILDCARD) || resource.table == WILDCARD {
                wildcard_resources.insert(resource.id, resource);
            } else {
                resources.insert(resource.clone().into(), resource);
            }
        }

        let mut permissions: HashMap<PermissionRequest, Permission> = Default::default();
        let mut wildcard_permissions = HashMap::new();
        for permission in permissions_rows {
            if permission.action == WILDCARD {
                wildcard_permissions.insert(permission.id, permission);
            } else {
                permissions.insert(permission.clone().into(), permission);
            }
        }

        let roles: HashMap<RoleId, Role> = roles_rows.into_iter().map(|r| (r.id, r)).collect();

        let mut user_roles: HashMap<UserId, RoleId> = Default::default();
        for user_role in user_roles_rows {
            user_roles.insert(user_role.user_id, user_role.role_id);
        }

        let mut role_permissions: HashMap<RoleId, HashSet<(PermissionId, ResourceId)>> =
            Default::default();
        for rp in role_permissions_rows {
            let set = role_permissions.entry(rp.role_id).or_default();
            set.insert((rp.permission_id, rp.resource_id));
        }

        let mut user_overrides: HashMap<UserId, Vec<UserOverride>> = Default::default();
        for user_override in user_overrides_rows {
            user_overrides
                .entry(user_override.user_id)
                .or_default()
                .push(user_override);
        }

        let mut role_hierarchy: HashMap<RoleId, Vec<RoleId>> = Default::default();
        for rh in role_hierarchy_rows {
            role_hierarchy
                .entry(rh.super_role_id)
                .or_default()
                .push(rh.role_id);
        }

        RbacEngine {
            resources,
            permissions,
            wildcard_resources,
            wildcard_permissions,
            roles,
            user_roles,
            role_permissions,
            user_overrides,
            role_hierarchy,
        }
    }

    fn get_user_role_ids(&self, user_id: &UserId) -> Result<HashSet<RoleId>, Error> {
        if let Some(role) = self.user_roles.get(&user_id) {
            let mut user_roles = HashSet::new();
            for role in enumerate_role(*role, &self.role_hierarchy) {
                if !self.roles.contains_key(&role) {
                    return Err(Error::RoleNotFound(format!("{role:?}")));
                }
                user_roles.insert(role);
            }
            Ok(user_roles)
        } else {
            Err(Error::UserNotFound(format!("{user_id:?}")))
        }
    }

    pub fn get_user_role_permissions(
        &self,
        user_id: UserId,
    ) -> Result<RbacUserRolePermissions, Error> {
        let mut user_roles: Vec<RoleId> = self.get_user_role_ids(&user_id)?.into_iter().collect();
        user_roles.sort();

        let mut role_permissions: HashSet<(PermissionId, ResourceId)> = Default::default();

        for role_id in user_roles {
            if let Some(items) = self.role_permissions.get(&role_id) {
                role_permissions.extend(items.into_iter());
            }
        }

        if let Some(user_overrides) = self.user_overrides.get(&user_id) {
            for over in user_overrides {
                let role_permission = (over.permission_id, over.resource_id);
                if role_permissions.contains(&role_permission) {
                    if !over.grant {
                        role_permissions.remove(&role_permission);
                    }
                } else if over.grant {
                    role_permissions.insert(role_permission);
                }
            }
        }

        let mut permissions = role_permissions
            .into_iter()
            .map(|(permission_id, resource_id)| {
                let resource = if let Some(r) = self.wildcard_resources.get(&resource_id) {
                    r
                } else {
                    self.resources
                        .values()
                        .find(|r| r.id == resource_id)
                        .ok_or_else(|| Error::ResourceNotFound(format!("{resource_id:?}")))?
                }
                .clone();
                let permission = if let Some(p) = self.wildcard_permissions.get(&permission_id) {
                    p
                } else {
                    self.permissions
                        .values()
                        .find(|p| p.id == permission_id)
                        .ok_or_else(|| Error::PermissionNotFound(format!("{permission_id:?}")))?
                }
                .clone();

                Ok((resource, permission))
            })
            .collect::<Result<Vec<_>, Error>>()?;

        permissions.sort_by_key(|r| (r.0.id, r.1.id));

        Ok(RbacUserRolePermissions {
            role: self
                .roles
                .get(&self.user_roles.get(&user_id).expect("Checked above"))
                .expect("Checked above")
                .to_owned(),
            permissions,
        })
    }

    pub fn user_can<P, R>(&self, user_id: UserId, permission: P, resource: R) -> Result<bool, Error>
    where
        P: Into<PermissionRequest>,
        R: Into<ResourceRequest>,
    {
        let resource = resource.into();
        let permission = permission.into();
        let resource = self.resources.get(&resource);
        let permission = self.permissions.get(&permission);

        // get user roles and flatten hierarchy
        let user_roles = self.get_user_role_ids(&user_id)?;

        if let (Some(permission), Some(resource)) = (permission, resource) {
            if let Some(user_overrides) = self.user_overrides.get(&user_id) {
                for user_override in user_overrides {
                    if user_override.permission_id == permission.id
                        && user_override.resource_id == resource.id
                    {
                        return Ok(user_override.grant);
                    }
                }
            }
        }

        for role_id in user_roles {
            if let Some(role_permissions) = self.role_permissions.get(&role_id) {
                if let (Some(permission), Some(resource)) = (permission, resource) {
                    if role_permissions.contains(&(permission.id, resource.id)) {
                        return Ok(true);
                    }
                }
                for (permission_id, resource_id) in role_permissions {
                    let is_wildcard_permission =
                        self.is_wildcard_permission(*permission_id, permission);
                    let is_wildcard_resource = self.is_wildcard_resource(*resource_id, resource);
                    if let Some(resource) = &resource {
                        if resource_id == &resource.id && is_wildcard_permission {
                            return Ok(true);
                        }
                    }
                    if let Some(permission) = &permission {
                        if permission_id == &permission.id && is_wildcard_resource {
                            return Ok(true);
                        }
                    }
                    if is_wildcard_permission && is_wildcard_resource {
                        return Ok(true);
                    }
                }
            }
        }

        if resource.is_none() {
            return Err(Error::ResourceNotFound(format!("{resource:?}")));
        }

        if permission.is_none() {
            return Err(Error::PermissionNotFound(format!("{permission:?}")));
        }

        Ok(false)
    }

    fn is_wildcard_resource(&self, id: ResourceId, target: Option<&Resource>) -> bool {
        if let Some(resource) = self.wildcard_resources.get(&id) {
            if let Some(target) = target {
                let schema_match = resource.schema.is_none()
                    || resource.schema.as_ref().unwrap() == WILDCARD
                    || resource.schema == target.schema;
                let table_match = resource.table == WILDCARD || resource.table == target.table;
                schema_match && table_match
            } else {
                (resource.schema.is_none() || resource.schema.as_ref().unwrap() == WILDCARD)
                    && resource.table == WILDCARD
            }
        } else {
            false
        }
    }

    fn is_wildcard_permission(&self, id: PermissionId, _: Option<&Permission>) -> bool {
        if let Some(permission) = self.wildcard_permissions.get(&id) {
            return permission.action == WILDCARD;
        }
        false
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[allow(non_snake_case)]
    fn Object(r: &str) -> Table<'_> {
        Table(r)
    }

    fn resource(table: &str) -> Resource {
        Resource {
            id: ResourceId(0),
            schema: None,
            table: table.to_owned(),
        }
    }

    fn permission(action: &str) -> Permission {
        Permission {
            id: PermissionId(0),
            action: action.to_owned(),
        }
    }

    fn role(role: &str) -> Role {
        Role {
            id: RoleId(0),
            role: role.to_owned(),
        }
    }

    fn seed_1() -> RbacSnapshot {
        let mut snapshot = RbacSnapshot::default();
        snapshot.set_resources(vec![
            resource("book"),
            resource("paper"),
            resource("pen"),
            resource("*"),
        ]);
        snapshot.set_permissions(vec![
            permission("browse"),  // read
            permission("buy"),     // create
            permission("replace"), // update
            permission("dispose"), // delete
            permission("*"),       // anything
        ]);
        snapshot.set_roles(vec![
            role("admin"),
            role("manager"),
            role("clerk"),
            role("auditor"),
        ]);
        snapshot.set_user_role(UserId(1), "admin");
        snapshot.set_user_role(UserId(2), "manager");
        snapshot.set_user_role(UserId(3), "clerk");
        snapshot.set_user_role(UserId(4), "auditor");
        snapshot.set_user_role(UserId(5), "clerk");

        snapshot.add_role_hierarchy("manager", "admin");
        snapshot.add_role_hierarchy("clerk", "manager");
        snapshot.add_role_hierarchy("auditor", "admin");

        snapshot.add_role_permission("clerk", Action("browse"), Object("pen"));
        snapshot.add_role_permission("clerk", Action("browse"), Object("paper"));
        snapshot.add_role_permission("clerk", Action("dispose"), Object("paper"));

        snapshot.add_role_permission("manager", Action("browse"), Object("book"));
        snapshot.add_role_permission("manager", Action("buy"), Object("book"));
        snapshot.add_role_permission("manager", Action("dispose"), Object("book"));
        snapshot.add_role_permission("manager", Action("replace"), Object("paper"));

        snapshot.add_role_permission("auditor", Action("browse"), Object("*"));

        snapshot.add_user_override(UserId(5), Action("buy"), Object("pen"), true);
        snapshot.add_user_override(UserId(5), Action("dispose"), Object("paper"), false);

        snapshot.add_role_permission("admin", Action("*"), Object("*"));

        snapshot
    }

    #[test]
    #[rustfmt::skip]
    fn test_rbac_engine_basic() {
        let admin = UserId(1);
        let manager = UserId(2);
        let clerk = UserId(3);
        let auditor = UserId(4);
        let designer = UserId(5);

        let engine = RbacEngine::from_snapshot(seed_1());

        // anyone can use pen and paper
        for item in ["pen", "paper"] {
            assert!(engine.user_can(clerk, Action("browse"), Object(item)).unwrap());
            assert!(engine.user_can(manager, Action("browse"), Object(item)).unwrap());
            assert!(engine.user_can(admin, Action("browse"), Object(item)).unwrap());
            // auditor can browse anything
            assert!(engine.user_can(auditor, Action("browse"), Object(item)).unwrap());
        }

        // anyone can dispose paper except auditor and designer
        for user in [clerk, manager, admin] {
            assert!(engine.user_can(user, Action("dispose"), Object("paper")).unwrap());
        }
        for user in [designer, auditor] {
            assert!(!engine.user_can(user, Action("dispose"), Object("paper")).unwrap());
        }

        // clerk cannot browse books
        for user in [clerk, designer] {
            assert!(!engine.user_can(user, Action("browse"), Object("book")).unwrap());
        }

        for user in [admin, manager] {
            assert!(engine.user_can(user, Action("browse"), Object("book")).unwrap());
            assert!(engine.user_can(user, Action("buy"), Object("book")).unwrap());
            assert!(engine.user_can(user, Action("dispose"), Object("book")).unwrap());
        }

        // auditor cannot alter things
        for action in ["buy", "replace", "dispose"] {
            for item in ["book", "paper", "pen"] {
                assert!(!engine.user_can(auditor, Action(action), Object(item)).unwrap());
            }
        }

        // manager cannot replace books, but admin can
        assert!(!engine.user_can(manager, Action("replace"), Object("book")).unwrap());
        assert!(engine.user_can(admin, Action("replace"), Object("book")).unwrap());

        // manager can replace paper
        assert!(!engine.user_can(clerk, Action("replace"), Object("paper")).unwrap());
        assert!(!engine.user_can(designer, Action("replace"), Object("paper")).unwrap());
        assert!(engine.user_can(manager, Action("replace"), Object("paper")).unwrap());
        assert!(engine.user_can(admin, Action("replace"), Object("paper")).unwrap());

        // only admin can buy paper
        for user in [clerk, manager, designer] {
            assert!(!engine.user_can(user, Action("buy"), Object("paper")).unwrap());
        }
        assert!(engine.user_can(admin, Action("buy"), Object("paper")).unwrap());

        // designer has an exception can buy pen
        for user in [designer, admin] {
            assert!(engine.user_can(user, Action("buy"), Object("pen")).unwrap());
        }
        for user in [clerk, manager] {
            assert!(!engine.user_can(user, Action("buy"), Object("pen")).unwrap());
        }

        // only admin can replace / dispose pen
        for action in ["replace", "dispose"] {
            assert!(engine.user_can(admin, Action(action), Object("pen")).unwrap());
        }

        // unknown action / object; admin has wildcard
        assert!(engine.user_can(admin, Action("?"), Object("?")).is_ok());
        assert!(engine.user_can(manager, Action("?"), Object("?")).is_err());
        assert!(engine.user_can(clerk, Action("?"), Object("?")).is_err());

        assert_eq!(engine.get_user_role_permissions(clerk).unwrap(), RbacUserRolePermissions {
            role: Role {
                id: RoleId(3),
                role: "clerk".to_owned(),
            },
            permissions: vec![
                (
                    Resource {
                        id: ResourceId(2),
                        schema: None,
                        table: "paper".to_owned(),
                    },
                    Permission {
                        id: PermissionId(1),
                        action: "browse".to_owned(),
                    },
                ),
                (
                    Resource {
                        id: ResourceId(2),
                        schema: None,
                        table: "paper".to_owned(),
                    },
                    Permission {
                        id: PermissionId(4),
                        action: "dispose".to_owned(),
                    },
                ),
                (
                    Resource {
                        id: ResourceId(3),
                        schema: None,
                        table: "pen".to_owned(),
                    },
                    Permission {
                        id: PermissionId(1),
                        action: "browse".to_owned(),
                    },
                ),
            ],
        });

        assert_eq!(engine.get_user_role_permissions(designer).unwrap(), RbacUserRolePermissions {
            role: Role {
                id: RoleId(3),
                role: "clerk".to_owned(),
            },
            permissions: vec![
                (
                    Resource {
                        id: ResourceId(2),
                        schema: None,
                        table: "paper".to_owned(),
                    },
                    Permission {
                        id: PermissionId(1),
                        action: "browse".to_owned(),
                    },
                ),
                (
                    Resource {
                        id: ResourceId(3),
                        schema: None,
                        table: "pen".to_owned(),
                    },
                    Permission {
                        id: PermissionId(1),
                        action: "browse".to_owned(),
                    },
                ),
                (
                    Resource {
                        id: ResourceId(3),
                        schema: None,
                        table: "pen".to_owned(),
                    },
                    Permission {
                        id: PermissionId(2),
                        action: "buy".to_owned(),
                    },
                ),
            ],
        });
    }

    #[rustfmt::skip]
    fn seed_2() -> RbacSnapshot {
        fn resource(schema: &str, table: &str) -> Resource {
            Resource {
                id: ResourceId(0),
                schema: Some(schema.to_owned()),
                table: table.to_owned(),
            }
        }

        let mut snapshot = RbacSnapshot::default();
        snapshot.set_resources(vec![
            resource("departmentA", "book"),
            resource("departmentB", "book"),
            resource("departmentB", "CD"),
            resource("*", "book"),
            resource("departmentB", "*"),
            resource("*", "*"),
        ]);
        snapshot.set_permissions(vec![
            permission("browse"),
        ]);
        snapshot.set_roles(vec![
            role("silver"),
            role("gold"),
            role("platinum"),
            role("reader"),
            role("admin"),
        ]);
        snapshot.set_user_role(UserId(1), "silver");
        snapshot.set_user_role(UserId(2), "gold");
        snapshot.set_user_role(UserId(3), "platinum");
        snapshot.set_user_role(UserId(4), "reader");
        snapshot.set_user_role(UserId(5), "admin");

        snapshot.add_role_permission("silver", Action("browse"), SchemaTable("departmentA", "book"));
        snapshot.add_role_permission("gold", Action("browse"), SchemaTable("departmentB", "book"));
        snapshot.add_role_permission("platinum", Action("browse"), SchemaTable("departmentA", "book"));
        snapshot.add_role_permission("platinum", Action("browse"), SchemaTable("departmentB", "*"));

        snapshot.add_role_permission("reader", Action("browse"), SchemaTable("*", "book"));

        snapshot.add_role_permission("admin", Action("browse"), SchemaTable("*", "*"));

        snapshot
    }

    #[test]
    #[rustfmt::skip]
    fn test_rbac_engine_wildcard() {
        let silver = UserId(1);
        let gold = UserId(2);
        let platinum = UserId(3);
        let reader = UserId(4);
        let admin = UserId(5);

        let engine = RbacEngine::from_snapshot(seed_2());

        assert!(engine.user_can(silver, Action("browse"), SchemaTable("departmentA", "book")).unwrap());
        assert!(!engine.user_can(silver, Action("browse"), SchemaTable("departmentB", "book")).unwrap());
        assert!(!engine.user_can(silver, Action("browse"), SchemaTable("departmentB", "CD")).unwrap());

        assert!(!engine.user_can(gold, Action("browse"), SchemaTable("departmentA", "book")).unwrap());
        assert!(engine.user_can(gold, Action("browse"), SchemaTable("departmentB", "book")).unwrap());
        assert!(!engine.user_can(gold, Action("browse"), SchemaTable("departmentB", "CD")).unwrap());

        assert!(engine.user_can(platinum, Action("browse"), SchemaTable("departmentA", "book")).unwrap());
        assert!(engine.user_can(platinum, Action("browse"), SchemaTable("departmentB", "book")).unwrap());
        assert!(engine.user_can(platinum, Action("browse"), SchemaTable("departmentB", "CD")).unwrap());

        assert!(engine.user_can(reader, Action("browse"), SchemaTable("departmentA", "book")).unwrap());
        assert!(engine.user_can(reader, Action("browse"), SchemaTable("departmentB", "book")).unwrap());
        assert!(!engine.user_can(reader, Action("browse"), SchemaTable("departmentB", "CD")).unwrap());

        assert!(engine.user_can(admin, Action("browse"), SchemaTable("departmentA", "book")).unwrap());
        assert!(engine.user_can(admin, Action("browse"), SchemaTable("departmentB", "book")).unwrap());
        assert!(engine.user_can(admin, Action("browse"), SchemaTable("departmentB", "CD")).unwrap());
    }

    #[rustfmt::skip]
    fn seed_3() -> RbacSnapshot {
        let mut snapshot = RbacSnapshot::default();
        snapshot.set_resources(vec![
            resource("book"),
            resource("CD"),
            resource("magazine"),
        ]);
        snapshot.set_permissions(vec![
            permission("browse"),
        ]);
        snapshot.set_roles(vec![
            role("A"),
            role("B"),
            role("C"),
            role("A+B"),
            role("A+C"),
            role("A+B+C"),
            role("(A+B)+C"),
        ]);
        snapshot.set_user_role(UserId(1), "A");
        snapshot.set_user_role(UserId(2), "B");
        snapshot.set_user_role(UserId(3), "C");
        snapshot.set_user_role(UserId(4), "A+B");
        snapshot.set_user_role(UserId(5), "A+C");
        snapshot.set_user_role(UserId(6), "A+B+C");
        snapshot.set_user_role(UserId(7), "(A+B)+C");

        snapshot.add_role_permission("A", Action("browse"), Object("book"));
        snapshot.add_role_permission("B", Action("browse"), Object("CD"));
        snapshot.add_role_permission("C", Action("browse"), Object("magazine"));

        snapshot.add_role_hierarchy("A", "A+B");
        snapshot.add_role_hierarchy("B", "A+B");

        snapshot.add_role_hierarchy("A", "A+C");
        snapshot.add_role_hierarchy("C", "A+C");

        snapshot.add_role_hierarchy("A", "A+B+C");
        snapshot.add_role_hierarchy("B", "A+B+C");
        snapshot.add_role_hierarchy("C", "A+B+C");

        snapshot.add_role_hierarchy("A+B", "(A+B)+C");
        snapshot.add_role_hierarchy("C", "(A+B)+C");

        snapshot
    }

    #[test]
    #[rustfmt::skip]
    #[allow(non_snake_case)]
    fn test_rbac_engine_hierarchy() {
        let A = UserId(1);
        let B = UserId(2);
        let C = UserId(3);
        let A_B = UserId(4);
        let A_C = UserId(5);
        let A_B_C = UserId(6);
        let A_B_C_ = UserId(7);

        let engine = RbacEngine::from_snapshot(seed_3());

        assert!(engine.user_can(A, Action("browse"), Object("book")).unwrap());
        assert!(!engine.user_can(A, Action("browse"), Object("CD")).unwrap());
        assert!(!engine.user_can(A, Action("browse"), Object("magazine")).unwrap());

        assert!(!engine.user_can(B, Action("browse"), Object("book")).unwrap());
        assert!(engine.user_can(B, Action("browse"), Object("CD")).unwrap());
        assert!(!engine.user_can(B, Action("browse"), Object("magazine")).unwrap());

        assert!(!engine.user_can(C, Action("browse"), Object("book")).unwrap());
        assert!(!engine.user_can(C, Action("browse"), Object("CD")).unwrap());
        assert!(engine.user_can(C, Action("browse"), Object("magazine")).unwrap());

        assert!(engine.user_can(A_B, Action("browse"), Object("book")).unwrap());
        assert!(engine.user_can(A_B, Action("browse"), Object("CD")).unwrap());
        assert!(!engine.user_can(A_B, Action("browse"), Object("magazine")).unwrap());
    
        assert!(engine.user_can(A_C, Action("browse"), Object("book")).unwrap());
        assert!(!engine.user_can(A_C, Action("browse"), Object("CD")).unwrap());
        assert!(engine.user_can(A_C, Action("browse"), Object("magazine")).unwrap());
    
        assert!(engine.user_can(A_B_C, Action("browse"), Object("book")).unwrap());
        assert!(engine.user_can(A_B_C, Action("browse"), Object("CD")).unwrap());
        assert!(engine.user_can(A_B_C, Action("browse"), Object("magazine")).unwrap());
    
        assert!(engine.user_can(A_B_C_, Action("browse"), Object("book")).unwrap());
        assert!(engine.user_can(A_B_C_, Action("browse"), Object("CD")).unwrap());
        assert!(engine.user_can(A_B_C_, Action("browse"), Object("magazine")).unwrap());
    }

    #[test]
    fn test_unrestricted() {
        let engine = RbacEngine::from_snapshot(RbacSnapshot::danger_unrestricted());
        assert_eq!(
            engine.get_user_role_permissions(UserId(0)).unwrap(),
            RbacUserRolePermissions {
                role: Role {
                    id: RoleId(1),
                    role: "unrestricted".to_owned(),
                },
                permissions: vec![(
                    Resource {
                        id: ResourceId(1),
                        schema: None,
                        table: "*".to_owned(),
                    },
                    Permission {
                        id: PermissionId(1),
                        action: "*".to_owned(),
                    },
                ),],
            }
        );
    }
}
