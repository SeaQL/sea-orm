use super::{
    AccessType, RbacError, RbacUserId,
    entity::{
        permission::{self, ActiveModel as Permission, PermissionId},
        resource::{self, ActiveModel as Resource, ResourceId},
        role::{self, ActiveModel as Role, RoleId},
        role_hierarchy::{self, ActiveModel as RoleHierarchy},
        role_permission::{self, ActiveModel as RolePermission},
        user_override::{self, ActiveModel as UserOverride},
        user_role::{self, ActiveModel as UserRole},
    },
};
use crate::{
    AccessMode, EntityTrait, IsolationLevel, Set, TransactionSession, TransactionTrait,
    error::DbErr, sea_query::OnConflict,
};
use std::collections::HashMap;

/// Helper class for manipulation of RBAC tables
#[derive(Debug)]
pub struct RbacContext {
    tables: HashMap<String, ResourceId>,
    permissions: HashMap<String, PermissionId>,
    roles: HashMap<String, RoleId>,
}

#[derive(Debug)]
pub struct RbacAddRoleHierarchy {
    pub super_role: &'static str,
    pub role: &'static str,
}

#[derive(Debug)]
pub struct RbacAddUserOverride {
    pub user_id: i64,
    pub table: &'static str,
    pub action: &'static str,
    pub grant: bool,
}

impl RbacContext {
    /// Load context from database connection
    pub async fn load<C: TransactionTrait>(db: &C) -> Result<Self, DbErr> {
        // ensure snapshot is consistent across all tables
        let txn = &db
            .begin_with_config(
                Some(IsolationLevel::ReadCommitted),
                Some(AccessMode::ReadOnly),
            )
            .await?;

        let tables = resource::Entity::find()
            .all(txn)
            .await?
            .into_iter()
            .map(|t| (t.table, t.id))
            .collect();

        let permissions = permission::Entity::find()
            .all(txn)
            .await?
            .into_iter()
            .map(|p| (p.action, p.id))
            .collect();

        let roles = role::Entity::find()
            .all(txn)
            .await?
            .into_iter()
            .map(|r| (r.role, r.id))
            .collect();

        Ok(Self {
            tables,
            permissions,
            roles,
        })
    }

    /// Add multiple tables as resources
    pub async fn add_tables<C: TransactionTrait>(
        &mut self,
        db: &C,
        tables: &[&'static str],
    ) -> Result<(), DbErr> {
        let txn = db.begin().await?;

        for table_name in tables {
            if let Some(table_id) = resource::Entity::insert(Resource {
                table: Set(table_name.to_string()),
                ..Default::default()
            })
            .on_conflict_do_nothing()
            .exec(&txn)
            .await?
            .last_insert_id()?
            {
                self.tables.insert(table_name.to_string(), table_id);
            }
        }

        txn.commit().await
    }

    /// Add CRUD actions
    pub async fn add_crud_permissions<C: TransactionTrait>(&mut self, db: &C) -> Result<(), DbErr> {
        let txn = db.begin().await?;

        for action in [
            AccessType::Select,
            AccessType::Insert,
            AccessType::Update,
            AccessType::Delete,
        ] {
            if let Some(permission_id) = permission::Entity::insert(Permission {
                action: Set(action.as_str().to_owned()),
                ..Default::default()
            })
            .on_conflict_do_nothing()
            .exec(&txn)
            .await?
            .last_insert_id()?
            {
                self.permissions
                    .insert(action.as_str().to_owned(), permission_id);
            }
        }

        txn.commit().await
    }

    /// Add multiple roles
    pub async fn add_roles<C: TransactionTrait>(
        &mut self,
        db: &C,
        roles: &[&'static str],
    ) -> Result<(), DbErr> {
        let txn = db.begin().await?;

        for role in roles {
            if let Some(role_id) = role::Entity::insert(Role {
                role: Set(role.to_string()),
                ..Default::default()
            })
            .on_conflict_do_nothing()
            .exec(&txn)
            .await?
            .last_insert_id()?
            {
                self.roles.insert(role.to_string(), role_id);
            }
        }

        txn.commit().await
    }

    pub fn get_role(&self, role: &'static str) -> Result<&RoleId, DbErr> {
        self.roles
            .get(role)
            .ok_or_else(|| DbErr::RbacError(RbacError::RoleNotFound(role.to_string()).to_string()))
    }

    /// Add permissions to roles. Will take cartesian product of tables and actions.
    pub async fn add_role_permissions<C: TransactionTrait>(
        &mut self,
        db: &C,
        role: &'static str,
        actions: &[&'static str],
        tables: &[&'static str],
    ) -> Result<(), DbErr> {
        self.update_role_permissions(db, role, actions, tables, true)
            .await
    }

    /// Remove permissions from roles. Will take cartesian product of tables and actions.
    pub async fn remove_role_permissions<C: TransactionTrait>(
        &mut self,
        db: &C,
        role: &'static str,
        actions: &[&'static str],
        tables: &[&'static str],
    ) -> Result<(), DbErr> {
        self.update_role_permissions(db, role, actions, tables, false)
            .await
    }

    async fn update_role_permissions<C: TransactionTrait>(
        &mut self,
        db: &C,
        role: &'static str,
        actions: &[&'static str],
        tables: &[&'static str],
        grant: bool,
    ) -> Result<(), DbErr> {
        let txn = db.begin().await?;

        for table_name in tables {
            for action in actions {
                let model = RolePermission {
                    role_id: Set(*self.roles.get(role).ok_or_else(|| {
                        DbErr::RbacError(RbacError::RoleNotFound(role.to_string()).to_string())
                    })?),
                    permission_id: Set(*self.permissions.get(*action).ok_or_else(|| {
                        DbErr::RbacError(
                            RbacError::PermissionNotFound(action.to_string()).to_string(),
                        )
                    })?),
                    resource_id: Set(*self.tables.get(*table_name).ok_or_else(|| {
                        DbErr::RbacError(
                            RbacError::ResourceNotFound(table_name.to_string()).to_string(),
                        )
                    })?),
                };
                if grant {
                    role_permission::Entity::insert(model)
                        .on_conflict_do_nothing()
                        .exec(&txn)
                        .await?;
                } else {
                    role_permission::Entity::delete(model).exec(&txn).await?;
                }
            }
        }

        txn.commit().await
    }

    pub async fn add_user_override<C: TransactionTrait>(
        &mut self,
        db: &C,
        rows: &[RbacAddUserOverride],
    ) -> Result<(), DbErr> {
        let txn = db.begin().await?;

        for RbacAddUserOverride {
            user_id,
            table,
            action,
            grant,
        } in rows
        {
            user_override::Entity::insert(UserOverride {
                user_id: Set(RbacUserId(*user_id)),
                permission_id: Set(*self.permissions.get(*action).ok_or_else(|| {
                    DbErr::RbacError(RbacError::PermissionNotFound(action.to_string()).to_string())
                })?),
                resource_id: Set(*self.tables.get(*table).ok_or_else(|| {
                    DbErr::RbacError(RbacError::ResourceNotFound(table.to_string()).to_string())
                })?),
                grant: Set(*grant),
            })
            .on_conflict(
                OnConflict::columns([
                    user_override::Column::UserId,
                    user_override::Column::PermissionId,
                    user_override::Column::ResourceId,
                ])
                .update_column(user_override::Column::Grant)
                .to_owned(),
            )
            .try_insert()
            .exec(&txn)
            .await?;
        }

        txn.commit().await
    }

    pub async fn add_role_hierarchy<C: TransactionTrait>(
        &mut self,
        db: &C,
        rows: &[RbacAddRoleHierarchy],
    ) -> Result<(), DbErr> {
        let txn = db.begin().await?;

        for RbacAddRoleHierarchy { super_role, role } in rows {
            role_hierarchy::Entity::insert(RoleHierarchy {
                super_role_id: Set(*self.roles.get(*super_role).ok_or_else(|| {
                    DbErr::RbacError(RbacError::RoleNotFound(super_role.to_string()).to_string())
                })?),
                role_id: Set(*self.roles.get(*role).ok_or_else(|| {
                    DbErr::RbacError(RbacError::RoleNotFound(role.to_string()).to_string())
                })?),
            })
            .on_conflict_do_nothing()
            .exec(&txn)
            .await?;
        }

        txn.commit().await
    }

    /// Assign role to users. Note that each user can only have 1 role,
    /// so this assignment replaces current role.
    /// `rows: (UserId, role)`
    pub async fn assign_user_role<C: TransactionTrait>(
        &mut self,
        db: &C,
        rows: &[(i64, &'static str)],
    ) -> Result<(), DbErr> {
        let txn = db.begin().await?;

        for (user_id, role) in rows {
            user_role::Entity::insert(UserRole {
                user_id: Set(RbacUserId(*user_id)),
                role_id: Set(*self.roles.get(*role).ok_or_else(|| {
                    DbErr::RbacError(RbacError::RoleNotFound(role.to_string()).to_string())
                })?),
            })
            .on_conflict(
                OnConflict::column(user_role::Column::UserId)
                    .update_column(user_role::Column::RoleId)
                    .to_owned(),
            )
            .try_insert()
            .exec(&txn)
            .await?;
        }

        txn.commit().await
    }
}
