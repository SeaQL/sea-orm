-- Schema for the typed-PK task tracker example.
--
-- Five tables, each chosen to cover one PK-newtype pattern not
-- already demonstrated by another table:
--
--   user                  -- a distinct PK type (`UserId`) so cross-entity
--                            confusion is rejectable at the type level.
--   project               -- a third distinct PK type (`ProjectId`),
--                            parent of task and project_member.
--   project_member        -- composite PK (ProjectId, UserId) with typed
--                            components.
--   task                  -- FK to project; two non-PK FKs to user
--                            (assignee + reviewer) that share the parent
--                            `UserId` type (codegen does NOT role-wrap
--                            non-PK FK columns, by design); self-ref
--                            `parent_task_id` for subtasks.
--   task_dependency       -- junction with two PK columns both FK to
--                            task.id. Codegen emits per-column role
--                            wrappers (`TaskDependencyPk*`) so swapping
--                            blocker/blocked at the call site fails to
--                            compile.
--
-- The entity files under src/entity/ are generated from this schema; see
-- Readme.md ("Regenerating the entities") for the sea-orm-cli command.

CREATE TABLE user (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE
);

CREATE TABLE project (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(255) NOT NULL
);

-- Composite PK whose components are both typed FKs into other tables.
CREATE TABLE project_member (
    project_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    role VARCHAR(64) NOT NULL,
    PRIMARY KEY (project_id, user_id),
    FOREIGN KEY (project_id) REFERENCES project (id),
    FOREIGN KEY (user_id) REFERENCES user (id)
);

-- Self-ref via parent_task_id; two non-PK FKs to user (assignee +
-- reviewer). Both user FKs share `UserId`, they are NOT role-wrapped
-- (role wrappers are PK-only by codegen design).
CREATE TABLE task (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL,
    assignee_id INTEGER NOT NULL,
    reviewer_id INTEGER,
    parent_task_id INTEGER,
    title VARCHAR(255) NOT NULL,
    FOREIGN KEY (project_id) REFERENCES project (id),
    FOREIGN KEY (assignee_id) REFERENCES user (id),
    FOREIGN KEY (reviewer_id) REFERENCES user (id),
    FOREIGN KEY (parent_task_id) REFERENCES task (id)
);

-- Junction with two PK columns both FK-referencing task.id. This is
-- the canonical role-wrapper case: codegen emits per-column wrapper
-- structs (`TaskDependencyPkBlockerTaskId`,
-- `TaskDependencyPkBlockedTaskId`) so positional swaps fail to
-- compile.
CREATE TABLE task_dependency (
    blocker_task_id INTEGER NOT NULL,
    blocked_task_id INTEGER NOT NULL,
    PRIMARY KEY (blocker_task_id, blocked_task_id),
    FOREIGN KEY (blocker_task_id) REFERENCES task (id),
    FOREIGN KEY (blocked_task_id) REFERENCES task (id)
);
