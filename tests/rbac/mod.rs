use super::*;
use crate::common::bakery_chain::*;
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, DbConn, EntityName, EntityTrait, ExecResult, Schema, Set,
    error::*,
    rbac::{
        AccessType, RbacUserId,
        entity::{
            permission::{ActiveModel as Permission, PermissionId},
            resource::{ActiveModel as Resource, ResourceId},
            role::{ActiveModel as Role, RoleId},
            role_hierarchy::ActiveModel as RoleHierarchy,
            role_permission::ActiveModel as RolePermission,
            user::UserId,
            user_override::ActiveModel as UserOverride,
            user_role::ActiveModel as UserRole,
        },
        schema::action_str,
    },
};
use std::collections::HashMap;

pub async fn setup(db: &DbConn) -> Result<(), DbErr> {
    let mut resources = HashMap::new();
    let mut permissions = HashMap::new();
    let mut roles = HashMap::new();

    let tables = [
        baker::Entity.table_name(),
        bakery::Entity.table_name(),
        cake::Entity.table_name(),
        cakes_bakers::Entity.table_name(),
        customer::Entity.table_name(),
        lineitem::Entity.table_name(),
        order::Entity.table_name(),
        "*", // WILDCARD
    ];

    for table_name in tables {
        resources.insert(
            table_name,
            Resource {
                table: Set(table_name.to_owned()),
                ..Default::default()
            }
            .insert(db)
            .await?
            .id,
        );
    }

    for action in [
        AccessType::Select,
        AccessType::Insert,
        AccessType::Update,
        AccessType::Delete,
    ] {
        permissions.insert(
            action_str(&action),
            Permission {
                action: Set(action_str(&action).to_owned()),
                ..Default::default()
            }
            .insert(db)
            .await?
            .id,
        );
    }

    for role in ["admin", "manager", "public"] {
        let role_id = Role {
            role: Set(role.to_owned()),
            ..Default::default()
        }
        .insert(db)
        .await?
        .id;
        roles.insert(role, role_id);

        UserRole {
            user_id: Set(RbacUserId(role_id.0)),
            role_id: Set(role_id),
        }
        .insert(db)
        .await?;
    }

    // public can select everything
    RolePermission {
        role_id: Set(*roles.get("public").unwrap()),
        permission_id: Set(*permissions.get("select").unwrap()),
        resource_id: Set(*resources.get("*").unwrap()),
    }
    .insert(db)
    .await?;

    // manager can create / update everything except bakery
    for (name, resource) in resources.iter() {
        if matches!(*name, "bakery" | "*") {
            continue;
        }
        for action in ["insert", "update"] {
            RolePermission {
                role_id: Set(*roles.get("manager").unwrap()),
                permission_id: Set(*permissions.get(action).unwrap()),
                resource_id: Set(*resource),
            }
            .insert(db)
            .await?;
        }
    }

    // manager can delete order
    for resource in ["order", "lineitem"] {
        RolePermission {
            role_id: Set(*roles.get("manager").unwrap()),
            permission_id: Set(*permissions.get("delete").unwrap()),
            resource_id: Set(*resources.get(resource).unwrap()),
        }
        .insert(db)
        .await?;
    }

    // admin can do anything, in addition to public / manager
    RolePermission {
        role_id: Set(*roles.get("admin").unwrap()),
        permission_id: Set(*permissions.get("delete").unwrap()),
        resource_id: Set(*resources.get("*").unwrap()),
    }
    .insert(db)
    .await?;

    // add permissions to bakery which manager doesn't have
    for action in ["insert", "update"] {
        RolePermission {
            role_id: Set(*roles.get("admin").unwrap()),
            permission_id: Set(*permissions.get(action).unwrap()),
            resource_id: Set(*resources.get("bakery").unwrap()),
        }
        .insert(db)
        .await?;
    }

    RoleHierarchy {
        role_id: Set(*roles.get("public").unwrap()),
        super_role_id: Set(*roles.get("manager").unwrap()),
    }
    .insert(db)
    .await?;
    RoleHierarchy {
        role_id: Set(*roles.get("manager").unwrap()),
        super_role_id: Set(*roles.get("admin").unwrap()),
    }
    .insert(db)
    .await?;

    Ok(())
}
