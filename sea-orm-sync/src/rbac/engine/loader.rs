use super::super::entity::{
    permission::Entity as Permission, resource::Entity as Resource, role::Entity as Role,
    role_hierarchy::Entity as RoleHierarchy, role_permission::Entity as RolePermission,
    user_override::Entity as UserOverride, user_role::Entity as UserRole,
};
use super::{RbacEngine, RbacSnapshot};
use crate::{AccessMode, DbConn, DbErr, EntityTrait, IsolationLevel, TransactionTrait};

impl RbacEngine {
    pub fn load_from(db: &DbConn) -> Result<Self, DbErr> {
        // ensure snapshot is consistent across all tables
        let txn = &db.begin_with_config(
            Some(IsolationLevel::ReadCommitted),
            Some(AccessMode::ReadOnly),
        )?;

        let resources = Resource::find().all(txn)?;
        let permissions = Permission::find().all(txn)?;
        let roles = Role::find().all(txn)?;
        let user_roles = UserRole::find().all(txn)?;
        let role_permissions = RolePermission::find().all(txn)?;
        let user_overrides = UserOverride::find().all(txn)?;
        let role_hierarchy = RoleHierarchy::find().all(txn)?;

        let snapshot = RbacSnapshot {
            resources,
            permissions,
            roles,
            user_roles,
            role_permissions,
            user_overrides,
            role_hierarchy,
        };

        Ok(Self::from_snapshot(snapshot))
    }
}
