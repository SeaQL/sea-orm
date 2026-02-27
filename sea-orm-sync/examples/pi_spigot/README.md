# Pi Spigot Algorithm Resumable Program Example

This example shows how easy it is to add SQLite-backed checkpointing to a
long-running computation using SeaORM. The idea applies to any stateful program:
batch jobs, data pipelines, simulations: anything you want to pause and resume.

We use the Rabinowitz-Wagon pi spigot algorithm as the example workload. It
streams decimal digits of pi one at a time, making it a perfect fit for
demonstrating incremental persistence.

## The Pattern: State Machine + Serialization

Any computation that can be modeled as a state machine can be made resumable.
The recipe has four parts:

```
new()        → initialize fresh state
step()       → advance one iteration, mutating &mut self
finalize()   → flush any buffered output
to_state()   → serialize self into a database row
from_state() → deserialize a database row back into self
```

### Step 1: Define your state as a SeaORM entity

Map every mutable field to a column:

```rust
pub mod state {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "state")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub digits: u32,         // identifies this computation
        pub boxes: JsonVec,      // algorithm working memory
        pub i: u32,              // current iteration
        pub nines: u32,          // buffered 9s
        pub predigit: u8,        // held digit
        pub have_predigit: bool,
        pub count: u32,          // digits emitted so far
        #[sea_orm(column_type = "Text")]
        pub result: String,      // emitted digits
    }
}
```

Complex types like `Vec<u32>` are stored as JSON columns via `FromJsonQueryResult`.

### Step 2: Write `to_state` / `from_state`

These convert between your in-memory struct and the entity model:

```rust
fn to_state(&self, i: u32) -> state::Model {
    state::Model {
        digits: self.digits,
        boxes: state::JsonVec(self.boxes.clone()),
        i,
        nines: self.nines,
        // ... every field
    }
}

fn from_state(s: state::Model) -> Self {
    Self {
        digits: s.digits,
        boxes: s.boxes.0,
        nines: s.nines,
        // ... every field
    }
}
```

### Step 3: Checkpoint with transactions

Inside your main loop, periodically save state. Use a transaction so the
checkpoint is atomic: either everything is saved or nothing is:

```rust
if self.count % checkpoint_interval == 0 {
    let txn = db.begin()?;
    state::Entity::delete_by_id(self.digits).exec(&txn)?;
    self.to_state(i + 1).into_active_model().insert(&txn)?;
    txn.commit()?;
}
```

### Step 4: Resume on startup

Check for an existing checkpoint. If found, reconstruct from it; otherwise
start fresh:

```rust
pub fn resume(db: &DatabaseConnection, digits: u32) -> Result<Self, DbErr> {
    db.get_schema_builder()
        .register(state::Entity)
        .sync(db)?;   // creates table if it doesn't exist

    match state::Entity::find_by_id(digits).one(db)? {
        Some(s) => Ok(Self::from_state(s)),
        None => Ok(Self::new(digits)),
    }
}
```

Note that `get_schema_builder().sync()` creates the table from the entity
definition automatically: no migrations needed.

## Running the example

```sh
# Compute 10000 digits of pi (checkpoints every 100 digits to pi.sqlite)
cargo run -- --digits 10000

# Press Ctrl-C at any time, then run again: it resumes from the last checkpoint
cargo run -- --digits 10000

# Use in-memory SQLite (no persistence, useful for testing)
cargo run -- --digits 1000 --db "sqlite::memory:"
```

## Running the tests

```sh
cargo test
```

The tests verify correctness against a known 1000-digit reference, including a
three-phase checkpoint/resume test: checkpoint at iteration 100, resume and
checkpoint again at 500, resume and finish at 1000.
