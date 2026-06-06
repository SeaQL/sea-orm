mod entity;
mod operations;

use entity::{project, project_member, task, user};
use sea_orm::{
    ActiveModelTrait, ActiveValue::*, ConnectOptions, ConnectionTrait, Database, DbBackend, DbErr,
    EntityTrait, Schema, Statement,
};

#[tokio::main]
async fn main() -> Result<(), DbErr> {
    let db = Database::connect(ConnectOptions::new("sqlite::memory:")).await?;
    create_schema(&db).await?;

    // Create two users. `.insert(...)` returns the persisted Model with
    // a typed `UserPk` already populated, no raw `i64` ever appears.
    let alice = user::ActiveModel {
        name: Set("Alice".to_string()),
        email: Set("alice@example.com".to_string()),
        ..Default::default()
    }
    .insert(&db)
    .await?;
    let bob = user::ActiveModel {
        name: Set("Bob".to_string()),
        email: Set("bob@example.com".to_string()),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    // Project + membership. `add_project_member` takes
    // `(ProjectId, UserId, String)`, swapping the two id args would be
    // a compile error.
    let audit = project::ActiveModel {
        name: Set("ATO compliance audit".to_string()),
        ..Default::default()
    }
    .insert(&db)
    .await?;
    operations::add_project_member(&db, audit.id, alice.id, "Admin".to_string()).await?;
    operations::add_project_member(&db, audit.id, bob.id, "Engineer".to_string()).await?;

    // Composite-PK lookup inline: `find_by_id` takes a tuple of typed
    // components. Reversing them is a compile error.
    let alice_membership = project_member::Entity::find_by_id((audit.id, alice.id))
        .one(&db)
        .await?
        .expect("alice should be a member");
    println!("alice's membership row: {alice_membership:?}");

    // Tasks. Each parameter to the typed insert is a distinct PK type;
    // a mixup would be rejected at compile time.
    let draft_policy = task::ActiveModel {
        project_id: Set(audit.id),
        assignee_id: Set(bob.id),
        reviewer_id: Set(Some(alice.id)),
        parent_task_id: Set(None),
        title: Set("Draft policy".to_string()),
        ..Default::default()
    }
    .insert(&db)
    .await?;
    println!("draft policy: {draft_policy:?}");

    // Subtask via self-ref. `parent_task_id: Set(Some(parent))` carries
    // a typed `TaskPk`, there's no way to accidentally pass a UserId.
    let outline =
        operations::create_subtask(&db, audit.id, draft_policy.id, bob.id, "Outline".to_string())
            .await?;
    println!("outline (subtask of draft policy): {outline:?}");

    let internal_review = task::ActiveModel {
        project_id: Set(audit.id),
        assignee_id: Set(alice.id),
        reviewer_id: Set(None),
        parent_task_id: Set(None),
        title: Set("Internal review".to_string()),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    // Role-wrapped junction. `add_blocker` funnels its two typed args
    // through distinct `TaskDependencyPk*` wrappers, so swapping the
    // blocker/blocked roles is a compile error at the insert site.
    operations::add_blocker(&db, draft_policy.id, internal_review.id).await?;
    println!("blocker recorded: \"draft policy\" blocks \"internal review\"");

    // Reassign outline from bob to alice. Typed PK threaded through UPDATE.
    let outline = operations::reassign_task(&db, outline.id, alice.id).await?;
    println!("outline reassigned to alice: {outline:?}");

    // Typed PK in `.filter()` position, a different code path than
    // `find_by_id`.
    let bobs_tasks = operations::tasks_assigned_to(&db, bob.id).await?;
    println!("tasks assigned to bob ({} total):", bobs_tasks.len());
    for t in &bobs_tasks {
        println!("  {t:?}");
    }

    // Composite-PK delete inline. Reversing the tuple components is a
    // compile error because `ProjectPk` and `UserPk` are distinct types.
    let removed = project_member::Entity::delete_by_id((audit.id, bob.id))
        .exec(&db)
        .await?;
    println!(
        "bob removed from project: {} row(s) affected",
        removed.rows_affected
    );

    Ok(())
}

/// Create the schema by running `CREATE TABLE` statements derived from
/// the entity definitions. For a real app you'd run migrations; for an
/// in-memory example this is the lightest path.
async fn create_schema(db: &impl ConnectionTrait) -> Result<(), DbErr> {
    let backend = db.get_database_backend();
    let schema = Schema::new(backend);

    // Order matters: parents before children so FKs resolve.
    for stmt in [
        schema.create_table_from_entity(user::Entity),
        schema.create_table_from_entity(project::Entity),
        schema.create_table_from_entity(project_member::Entity),
        schema.create_table_from_entity(task::Entity),
        schema.create_table_from_entity(entity::task_dependency::Entity),
    ] {
        db.execute(&stmt).await?;
    }
    // Enable FK enforcement on SQLite so the example actually exercises
    // the constraints.
    db.execute_raw(Statement::from_string(
        DbBackend::Sqlite,
        "PRAGMA foreign_keys = ON".to_string(),
    ))
    .await?;
    Ok(())
}
