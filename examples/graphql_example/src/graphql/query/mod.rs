use entity::async_graphql;

pub mod note;

pub use note::NoteQuery;

// Add your other ones here to create a unified Query object
// e.x. Query(NoteQuery, OtherQuery, OtherOtherQuery)
#[derive(async_graphql::MergedObject, Default)]
pub struct Query(NoteQuery);
