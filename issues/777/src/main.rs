use sea_orm::{DatabaseConnection, DbBackend, EntityTrait, MockDatabase, PrimaryKeyTrait, Related};

pub fn get_connection() -> DatabaseConnection {
    MockDatabase::new(DbBackend::Postgres).into_connection()
}

pub async fn get_entities<T>() -> Vec<<T as EntityTrait>::Model>
where
    T: EntityTrait,
{
    let connection = get_connection();
    T::find().all(&connection).await.unwrap_or_default()
}

pub async fn find_entity_by_id<T>(
    id_entity: <T::PrimaryKey as PrimaryKeyTrait>::ValueType,
) -> <T as EntityTrait>::Model
where
    T: EntityTrait,
{
    let connection = get_connection();
    T::find_by_id(id_entity)
        .one(&connection)
        .await
        .unwrap_or_default()
        .unwrap()
}

pub async fn find_entity_by_id_with_related<T, R>(
    id_entity: <T::PrimaryKey as PrimaryKeyTrait>::ValueType,
    r: R,
) where
    T: EntityTrait + Related<R>,
    R: EntityTrait,
{
    let connection = get_connection();
    let model = T::find_by_id(id_entity).find_with_related(r);
}

#[smol_potat::main]
async fn main() {}
