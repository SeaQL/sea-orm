# sea_orm_underscore_fields
A minimal repository showing an issue with SeaORM.

Connects to the database with `env!()`, so make sure to set `DATABASE_URL` when compiling.

The file `src/entity/underscores_workaround.rs` shows the workaround to get the names to query correctly, and what happens if it's not included.
