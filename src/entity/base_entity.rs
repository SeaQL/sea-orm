use crate::{
    ActiveModelTrait, ColumnTrait, Delete, DeleteMany, DeleteOne, FromQueryResult, Insert,
    ModelTrait, PrimaryKeyToColumn, PrimaryKeyTrait, QueryFilter, Related, RelationBuilder,
    RelationTrait, RelationType, Select, Update, UpdateMany, UpdateOne,
};
use sea_query::{Alias, Iden, IntoIden, IntoTableRef, IntoValueTuple, TableRef};
pub use sea_strum::IntoEnumIterator as Iterable;
use std::fmt::Debug;

/// Ensure the identifier for an Entity can be converted to a static str
pub trait IdenStatic: Iden + Copy + Debug + 'static {
    /// Method to call to get the static string identity
    fn as_str(&self) -> &str;
}

/// A Trait for mapping an Entity to a database table
pub trait EntityName: IdenStatic + Default {
    /// Method to get the name for the schema, defaults to [Option::None] if not set
    fn schema_name(&self) -> Option<&str> {
        None
    }

    /// Get the name of the table
    fn table_name(&self) -> &str;

    /// Get the name of the module from the invoking `self.table_name()`
    fn module_name(&self) -> &str {
        self.table_name()
    }

    /// Get the [TableRef] from invoking the `self.schema_name()`
    fn table_ref(&self) -> TableRef {
        match self.schema_name() {
            Some(schema) => (Alias::new(schema).into_iden(), self.into_iden()).into_table_ref(),
            None => self.into_table_ref(),
        }
    }
}

/// An Entity implementing `EntityTrait` represents a table in a database.
///
/// This trait provides an API for you to inspect it's properties
/// - Column (implemented [`ColumnTrait`])
/// - Relation (implemented [`RelationTrait`])
/// - Primary Key (implemented [`PrimaryKeyTrait`] and [`PrimaryKeyToColumn`])
///
/// This trait also provides an API for CRUD actions
/// - Select: `find`, `find_*`
/// - Insert: `insert`, `insert_*`
/// - Update: `update`, `update_*`
/// - Delete: `delete`, `delete_*`
pub trait EntityTrait: EntityName {
    #[allow(missing_docs)]
    type Model: ModelTrait<Entity = Self> + FromQueryResult;

    #[allow(missing_docs)]
    type Column: ColumnTrait;

    #[allow(missing_docs)]
    type Relation: RelationTrait;

    #[allow(missing_docs)]
    type PrimaryKey: PrimaryKeyTrait + PrimaryKeyToColumn<Column = Self::Column>;

    /// Check if the relation belongs to an Entity
    fn belongs_to<R>(related: R) -> RelationBuilder<Self, R>
    where
        R: EntityTrait,
    {
        RelationBuilder::new(RelationType::HasOne, Self::default(), related, false)
    }

    /// Check if the entity has at least one relation
    fn has_one<R>(_: R) -> RelationBuilder<Self, R>
    where
        R: EntityTrait + Related<Self>,
    {
        RelationBuilder::from_rel(RelationType::HasOne, R::to().rev(), true)
    }

    /// Check if the Entity has many relations
    fn has_many<R>(_: R) -> RelationBuilder<Self, R>
    where
        R: EntityTrait + Related<Self>,
    {
        RelationBuilder::from_rel(RelationType::HasMany, R::to().rev(), true)
    }

    /// Construct select statement to find one / all models
    ///
    /// - To select columns, join tables and group by expressions, see [`QuerySelect`](crate::query::QuerySelect)
    /// - To apply where conditions / filters, see [`QueryFilter`](crate::query::QueryFilter)
    /// - To apply order by expressions, see [`QueryOrder`](crate::query::QueryOrder)
    ///
    /// # Example
    ///
    /// ```
    /// # use sea_orm::{error::*, tests_cfg::*, *};
    /// #
    /// # #[smol_potat::main]
    /// # #[cfg(feature = "mock")]
    /// # pub async fn main() -> Result<(), DbErr> {
    /// #
    /// # let db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_query_results(vec![
    /// #         vec![
    /// #             cake::Model {
    /// #                 id: 1,
    /// #                 name: "New York Cheese".to_owned(),
    /// #             },
    /// #         ],
    /// #         vec![
    /// #             cake::Model {
    /// #                 id: 1,
    /// #                 name: "New York Cheese".to_owned(),
    /// #             },
    /// #             cake::Model {
    /// #                 id: 2,
    /// #                 name: "Chocolate Forest".to_owned(),
    /// #             },
    /// #         ],
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake};
    ///
    /// assert_eq!(
    ///     cake::Entity::find().one(&db).await?,
    ///     Some(cake::Model {
    ///         id: 1,
    ///         name: "New York Cheese".to_owned(),
    ///     })
    /// );
    ///
    /// assert_eq!(
    ///     cake::Entity::find().all(&db).await?,
    ///     vec![
    ///         cake::Model {
    ///             id: 1,
    ///             name: "New York Cheese".to_owned(),
    ///         },
    ///         cake::Model {
    ///             id: 2,
    ///             name: "Chocolate Forest".to_owned(),
    ///         },
    ///     ]
    /// );
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![
    ///         Transaction::from_sql_and_values(
    ///             DbBackend::Postgres,
    ///             r#"SELECT "cake"."id", "cake"."name" FROM "cake" LIMIT $1"#,
    ///             vec![1u64.into()]
    ///         ),
    ///         Transaction::from_sql_and_values(
    ///             DbBackend::Postgres,
    ///             r#"SELECT "cake"."id", "cake"."name" FROM "cake""#,
    ///             vec![]
    ///         ),
    ///     ]
    /// );
    /// #
    /// # Ok(())
    /// # }
    /// ```
    fn find() -> Select<Self> {
        Select::new()
    }

    /// Find a model by primary key
    ///
    /// # Example
    ///
    /// ```
    /// # use sea_orm::{error::*, tests_cfg::*, *};
    /// #
    /// # #[smol_potat::main]
    /// # #[cfg(feature = "mock")]
    /// # pub async fn main() -> Result<(), DbErr> {
    /// #
    /// # let db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_query_results(vec![
    /// #         vec![
    /// #             cake::Model {
    /// #                 id: 11,
    /// #                 name: "Sponge Cake".to_owned(),
    /// #             },
    /// #         ],
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake};
    ///
    /// assert_eq!(
    ///     cake::Entity::find_by_id(11).all(&db).await?,
    ///     vec![cake::Model {
    ///         id: 11,
    ///         name: "Sponge Cake".to_owned(),
    ///     }]
    /// );
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"SELECT "cake"."id", "cake"."name" FROM "cake" WHERE "cake"."id" = $1"#,
    ///         vec![11i32.into()]
    ///     )]
    /// );
    /// #
    /// # Ok(())
    /// # }
    /// ```
    /// Find by composite key
    /// ```
    /// # use sea_orm::{error::*, tests_cfg::*, *};
    /// #
    /// # #[smol_potat::main]
    /// # #[cfg(feature = "mock")]
    /// # pub async fn main() -> Result<(), DbErr> {
    /// #
    /// # let db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_query_results(vec![
    /// #         vec![
    /// #             cake_filling::Model {
    /// #                 cake_id: 2,
    /// #                 filling_id: 3,
    /// #             },
    /// #         ],
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake_filling};
    ///
    /// assert_eq!(
    ///     cake_filling::Entity::find_by_id((2, 3)).all(&db).await?,
    ///     vec![cake_filling::Model {
    ///         cake_id: 2,
    ///         filling_id: 3,
    ///     }]
    /// );
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         [
    ///             r#"SELECT "cake_filling"."cake_id", "cake_filling"."filling_id" FROM "cake_filling""#,
    ///             r#"WHERE "cake_filling"."cake_id" = $1 AND "cake_filling"."filling_id" = $2"#,
    ///         ].join(" ").as_str(),
    ///         vec![2i32.into(), 3i32.into()]
    ///     )]);
    /// #
    /// # Ok(())
    /// # }
    /// ```
    fn find_by_id(values: <Self::PrimaryKey as PrimaryKeyTrait>::ValueType) -> Select<Self> {
        let mut select = Self::find();
        let mut keys = Self::PrimaryKey::iter();
        for v in values.into_value_tuple() {
            if let Some(key) = keys.next() {
                let col = key.into_column();
                select = select.filter(col.eq(v));
            } else {
                panic!("primary key arity mismatch");
            }
        }
        if keys.next().is_some() {
            panic!("primary key arity mismatch");
        }
        select
    }

    /// Insert an model into database
    ///
    /// # Example (Postgres)
    ///
    /// ```
    /// # use sea_orm::{error::*, tests_cfg::*, *};
    /// #
    /// # #[smol_potat::main]
    /// # #[cfg(feature = "mock")]
    /// # pub async fn main() -> Result<(), DbErr> {
    /// #
    /// # let db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_exec_results(vec![
    /// #         MockExecResult {
    /// #             last_insert_id: 15,
    /// #             rows_affected: 1,
    /// #         },
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake};
    ///
    /// let apple = cake::ActiveModel {
    ///     name: Set("Apple Pie".to_owned()),
    ///     ..Default::default()
    /// };
    ///
    /// let insert_result = cake::Entity::insert(apple).exec(&db).await?;
    ///
    /// assert_eq!(dbg!(insert_result.last_insert_id), 150);
    /// assert!(false);
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"INSERT INTO "cake" ("name") VALUES ($1) RETURNING "id""#,
    ///         vec!["Apple Pie".into()]
    ///     )]);
    /// #
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Example (MySQL)
    ///
    /// ```
    /// # use sea_orm::{error::*, tests_cfg::*, *};
    /// #
    /// # #[smol_potat::main]
    /// # #[cfg(feature = "mock")]
    /// # pub async fn main() -> Result<(), DbErr> {
    /// #
    /// # let db = MockDatabase::new(DbBackend::MySql)
    /// #     .append_exec_results(vec![
    /// #         MockExecResult {
    /// #             last_insert_id: 15,
    /// #             rows_affected: 1,
    /// #         },
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake};
    ///
    /// let apple = cake::ActiveModel {
    ///     name: Set("Apple Pie".to_owned()),
    ///     ..Default::default()
    /// };
    ///
    /// let insert_result = cake::Entity::insert(apple).exec(&db).await?;
    ///
    /// assert_eq!(insert_result.last_insert_id, 15);
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::MySql,
    ///         r#"INSERT INTO `cake` (`name`) VALUES (?)"#,
    ///         vec!["Apple Pie".into()]
    ///     )]);
    /// #
    /// # Ok(())
    /// # }
    /// ```
    fn insert<A>(model: A) -> Insert<A>
    where
        A: ActiveModelTrait<Entity = Self>,
    {
        Insert::one(model)
    }

    /// Insert many models into database
    ///
    /// # Example (Postgres)
    ///
    /// ```
    /// # use sea_orm::{error::*, tests_cfg::*, *};
    /// #
    /// # #[smol_potat::main]
    /// # #[cfg(feature = "mock")]
    /// # pub async fn main() -> Result<(), DbErr> {
    /// #
    /// # let db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_exec_results(vec![
    /// #         MockExecResult {
    /// #             last_insert_id: 28,
    /// #             rows_affected: 2,
    /// #         },
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake};
    ///
    /// let apple = cake::ActiveModel {
    ///     name: Set("Apple Pie".to_owned()),
    ///     ..Default::default()
    /// };
    /// let orange = cake::ActiveModel {
    ///     name: Set("Orange Scone".to_owned()),
    ///     ..Default::default()
    /// };
    ///
    /// let insert_result = cake::Entity::insert_many(vec![apple, orange]).exec(&db).await?;
    ///
    /// assert_eq!(insert_result.last_insert_id, 28);
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"INSERT INTO "cake" ("name") VALUES ($1), ($2) RETURNING "id""#,
    ///         vec!["Apple Pie".into(), "Orange Scone".into()]
    ///     )]);
    /// #
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Example (MySQL)
    ///
    /// ```
    /// # use sea_orm::{error::*, tests_cfg::*, *};
    /// #
    /// # #[smol_potat::main]
    /// # #[cfg(feature = "mock")]
    /// # pub async fn main() -> Result<(), DbErr> {
    /// #
    /// # let db = MockDatabase::new(DbBackend::MySql)
    /// #     .append_exec_results(vec![
    /// #         MockExecResult {
    /// #             last_insert_id: 28,
    /// #             rows_affected: 2,
    /// #         },
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake};
    ///
    /// let apple = cake::ActiveModel {
    ///     name: Set("Apple Pie".to_owned()),
    ///     ..Default::default()
    /// };
    /// let orange = cake::ActiveModel {
    ///     name: Set("Orange Scone".to_owned()),
    ///     ..Default::default()
    /// };
    ///
    /// let insert_result = cake::Entity::insert_many(vec![apple, orange]).exec(&db).await?;
    ///
    /// assert_eq!(insert_result.last_insert_id, 28);
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::MySql,
    ///         r#"INSERT INTO `cake` (`name`) VALUES (?), (?)"#,
    ///         vec!["Apple Pie".into(), "Orange Scone".into()]
    ///     )]);
    /// #
    /// # Ok(())
    /// # }
    /// ```
    fn insert_many<A, I>(models: I) -> Insert<A>
    where
        A: ActiveModelTrait<Entity = Self>,
        I: IntoIterator<Item = A>,
    {
        Insert::many(models)
    }

    /// Update an model in database
    ///
    /// - To apply where conditions / filters, see [`QueryFilter`](crate::query::QueryFilter)
    ///
    /// # Example (Postgres)
    ///
    /// ```
    /// # use sea_orm::{error::*, tests_cfg::*, *};
    /// #
    /// # #[smol_potat::main]
    /// # #[cfg(feature = "mock")]
    /// # pub async fn main() -> Result<(), DbErr> {
    /// #
    /// # let db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_exec_results(vec![
    /// #         MockExecResult {
    /// #             last_insert_id: 0,
    /// #             rows_affected: 1,
    /// #         },
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::fruit};
    ///
    /// let orange = fruit::ActiveModel {
    ///     id: Set(1),
    ///     name: Set("Orange".to_owned()),
    ///     ..Default::default()
    /// };
    ///
    /// assert_eq!(
    ///     fruit::Entity::update(orange.clone())
    ///         .filter(fruit::Column::Name.contains("orange"))
    ///         .exec(&db)
    ///         .await?,
    ///     orange
    /// );
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"UPDATE "fruit" SET "name" = $1 WHERE "fruit"."id" = $2 AND "fruit"."name" LIKE $3 RETURNING "id", "name", "cake_id""#,
    ///         vec!["Orange".into(), 1i32.into(), "%orange%".into()]
    ///     )]);
    /// #
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Example (MySQL)
    ///
    /// ```
    /// # use sea_orm::{error::*, tests_cfg::*, *};
    /// #
    /// # #[smol_potat::main]
    /// # #[cfg(feature = "mock")]
    /// # pub async fn main() -> Result<(), DbErr> {
    /// #
    /// # let db = MockDatabase::new(DbBackend::MySql)
    /// #     .append_query_results(vec![
    /// #         vec![fruit::Model {
    /// #             id: 1,
    /// #             name: "Orange".to_owned(),
    /// #             cake_id: None,
    /// #         }],
    /// #     ])
    /// #     .append_exec_results(vec![
    /// #         MockExecResult {
    /// #             last_insert_id: 0,
    /// #             rows_affected: 1,
    /// #         },
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::fruit};
    ///
    /// let orange = fruit::ActiveModel {
    ///     id: Set(1),
    ///     name: Set("Orange".to_owned()),
    ///     ..Default::default()
    /// };
    ///
    /// assert_eq!(
    ///     fruit::Entity::update(orange.clone())
    ///         .filter(fruit::Column::Name.contains("orange"))
    ///         .exec(&db)
    ///         .await?,
    ///     orange
    /// );
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![
    ///         Transaction::from_sql_and_values(
    ///             DbBackend::MySql,
    ///             r#"UPDATE `fruit` SET `name` = ? WHERE `fruit`.`id` = ? AND `fruit`.`name` LIKE ?"#,
    ///             vec!["Orange".into(), 1i32.into(), "%orange%".into()]
    ///         ),
    ///         Transaction::from_sql_and_values(
    ///             DbBackend::MySql,
    ///             r#"SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit` WHERE `fruit`.`id` = ? LIMIT ?"#,
    ///             vec![1i32.into(), 1u64.into()]
    ///         )]);
    /// #
    /// # Ok(())
    /// # }
    /// ```
    fn update<A>(model: A) -> UpdateOne<A>
    where
        A: ActiveModelTrait<Entity = Self>,
    {
        Update::one(model)
    }

    /// Update many models in database
    ///
    /// - To apply where conditions / filters, see [`QueryFilter`](crate::query::QueryFilter)
    ///
    /// # Example
    ///
    /// ```
    /// # use sea_orm::{error::*, tests_cfg::*, *};
    /// #
    /// # #[smol_potat::main]
    /// # #[cfg(feature = "mock")]
    /// # pub async fn main() -> Result<(), DbErr> {
    /// #
    /// # let db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_exec_results(vec![
    /// #         MockExecResult {
    /// #             last_insert_id: 0,
    /// #             rows_affected: 5,
    /// #         },
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::fruit, sea_query::{Expr, Value}};
    ///
    /// let update_result = fruit::Entity::update_many()
    ///     .col_expr(fruit::Column::CakeId, Expr::value(Value::Int(None)))
    ///     .filter(fruit::Column::Name.contains("Apple"))
    ///     .exec(&db)
    ///     .await?;
    ///
    /// assert_eq!(update_result.rows_affected, 5);
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"UPDATE "fruit" SET "cake_id" = $1 WHERE "fruit"."name" LIKE $2"#,
    ///         vec![Value::Int(None), "%Apple%".into()]
    ///     )]);
    /// #
    /// # Ok(())
    /// # }
    /// ```
    fn update_many() -> UpdateMany<Self> {
        Update::many(Self::default())
    }

    /// Delete an model from database
    ///
    /// - To apply where conditions / filters, see [`QueryFilter`](crate::query::QueryFilter)
    ///
    /// # Example
    ///
    /// ```
    /// # use sea_orm::{error::*, tests_cfg::*, *};
    /// #
    /// # #[smol_potat::main]
    /// # #[cfg(feature = "mock")]
    /// # pub async fn main() -> Result<(), DbErr> {
    /// #
    /// # let db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_exec_results(vec![
    /// #         MockExecResult {
    /// #             last_insert_id: 0,
    /// #             rows_affected: 1,
    /// #         },
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::fruit};
    ///
    /// let orange = fruit::ActiveModel {
    ///     id: Set(3),
    ///     ..Default::default()
    /// };
    ///
    /// let delete_result = fruit::Entity::delete(orange).exec(&db).await?;
    ///
    /// assert_eq!(delete_result.rows_affected, 1);
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres, r#"DELETE FROM "fruit" WHERE "fruit"."id" = $1"#,
    ///         vec![3i32.into()]
    ///     )]);
    /// #
    /// # Ok(())
    /// # }
    /// ```
    fn delete<A>(model: A) -> DeleteOne<A>
    where
        A: ActiveModelTrait<Entity = Self>,
    {
        Delete::one(model)
    }

    /// Delete many models from database
    ///
    /// - To apply where conditions / filters, see [`QueryFilter`](crate::query::QueryFilter)
    ///
    /// # Example
    ///
    /// ```
    /// # use sea_orm::{error::*, tests_cfg::*, *};
    /// #
    /// # #[smol_potat::main]
    /// # #[cfg(feature = "mock")]
    /// # pub async fn main() -> Result<(), DbErr> {
    /// #
    /// # let db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_exec_results(vec![
    /// #         MockExecResult {
    /// #             last_insert_id: 0,
    /// #             rows_affected: 5,
    /// #         },
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::fruit};
    ///
    /// let delete_result = fruit::Entity::delete_many()
    ///     .filter(fruit::Column::Name.contains("Apple"))
    ///     .exec(&db)
    ///     .await?;
    ///
    /// assert_eq!(delete_result.rows_affected, 5);
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"DELETE FROM "fruit" WHERE "fruit"."name" LIKE $1"#,
    ///         vec!["%Apple%".into()]
    ///     )]);
    /// #
    /// # Ok(())
    /// # }
    /// ```
    fn delete_many() -> DeleteMany<Self> {
        Delete::many(Self::default())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(feature = "macros")]
    fn entity_model_1() {
        use crate::entity::*;

        mod hello {
            use crate as sea_orm;
            use crate::entity::prelude::*;

            #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
            #[sea_orm(table_name = "hello")]
            pub struct Model {
                #[sea_orm(primary_key)]
                pub id: i32,
            }

            #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
            pub enum Relation {}

            impl ActiveModelBehavior for ActiveModel {}
        }

        assert_eq!(hello::Entity.table_name(), "hello");
        assert_eq!(hello::Entity.schema_name(), None);
    }

    #[test]
    #[cfg(feature = "macros")]
    fn entity_model_2() {
        use crate::entity::*;

        mod hello {
            use crate as sea_orm;
            use crate::entity::prelude::*;

            #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
            #[sea_orm(table_name = "hello", schema_name = "world")]
            pub struct Model {
                #[sea_orm(primary_key)]
                pub id: i32,
            }

            #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
            pub enum Relation {}

            impl ActiveModelBehavior for ActiveModel {}
        }

        assert_eq!(hello::Entity.table_name(), "hello");
        assert_eq!(hello::Entity.schema_name(), Some("world"));
    }
}
