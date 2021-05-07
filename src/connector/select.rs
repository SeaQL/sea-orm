use crate::{Connection, Database, Entity, QueryErr, QueryResult, Select};

impl<E: 'static> Select<'_, E>
where
    E: Entity,
{
    pub async fn one(self, db: &Database) -> Result<QueryResult, QueryErr> {
        let builder = db.get_query_builder_backend();
        db.get_connection().query_one(self.build(builder)).await
    }

    pub async fn all(self, db: &Database) -> Result<Vec<QueryResult>, QueryErr> {
        let builder = db.get_query_builder_backend();
        db.get_connection().query_all(self.build(builder)).await
    }
}
