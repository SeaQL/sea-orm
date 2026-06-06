//! Typed-PK domain code over the generated task-tracker entities.
//!
//! Every function signature carries typed IDs, so the compiler catches
//! mixups at every call site. The trybuild fixtures under
//! `tests/value_type_pk_compile_fail/` pin the rejection contract;
//! this module is the positive side.
//!
//! Each function covers a distinct typed-PK call shape:
//!
//!   - `reassign_task`, typed PK threaded through an `UPDATE` and
//!     cross-entity argument typing (`TaskPk` and `UserPk`).
//!   - `create_subtask`, self-ref `parent_task_id: Option<TaskPk>`
//!     plus four typed PKs in a row.
//!   - `add_blocker`, role-wrapped junction insert (the only place
//!     the `TaskDependencyPk*` wrappers are user-visible).
//!   - `add_project_member`, composite-PK insert with typed components.
//!   - `tasks_assigned_to`, typed PK as a value passed into
//!     `Column::AssigneeId.eq(...)` for a filter.

use crate::entity::{project, project_member, task, task_dependency, user};
use sea_orm::{ActiveValue::*, DbErr, entity::*, query::*};

pub async fn reassign_task<C: ConnectionTrait>(
    db: &C,
    task_id: task::TaskPk,
    new_assignee: user::UserPk,
) -> Result<task::Model, DbErr> {
    let existing = task::Entity::find_by_id(task_id)
        .one(db)
        .await?
        .ok_or_else(|| DbErr::RecordNotFound(format!("task {task_id:?}")))?;
    let mut active: task::ActiveModel = existing.into();
    active.assignee_id = Set(new_assignee);
    active.update(db).await
}

pub async fn create_subtask<C: ConnectionTrait>(
    db: &C,
    project_id: project::ProjectPk,
    parent: task::TaskPk,
    assignee: user::UserPk,
    title: String,
) -> Result<task::Model, DbErr> {
    task::ActiveModel {
        project_id: Set(project_id),
        assignee_id: Set(assignee),
        reviewer_id: Set(None),
        parent_task_id: Set(Some(parent)),
        title: Set(title),
        ..Default::default()
    }
    .insert(db)
    .await
}

pub async fn add_blocker<C: ConnectionTrait>(
    db: &C,
    blocker: task::TaskPk,
    blocked: task::TaskPk,
) -> Result<task_dependency::Model, DbErr> {
    task_dependency::ActiveModel {
        blocker_task_id: Set(task_dependency::TaskDependencyPkBlockerTaskId(blocker)),
        blocked_task_id: Set(task_dependency::TaskDependencyPkBlockedTaskId(blocked)),
    }
    .insert(db)
    .await
}

pub async fn add_project_member<C: ConnectionTrait>(
    db: &C,
    project_id: project::ProjectPk,
    user_id: user::UserPk,
    role: String,
) -> Result<project_member::Model, DbErr> {
    project_member::ActiveModel {
        project_id: Set(project_id),
        user_id: Set(user_id),
        role: Set(role),
    }
    .insert(db)
    .await
}

pub async fn tasks_assigned_to<C: ConnectionTrait>(
    db: &C,
    user_id: user::UserPk,
) -> Result<Vec<task::Model>, DbErr> {
    task::Entity::find()
        .filter(task::Column::AssigneeId.eq(user_id))
        .all(db)
        .await
}
