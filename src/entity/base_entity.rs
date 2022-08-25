use crate::{
    ActiveModelTrait, ColumnTrait, Delete, DeleteMany, DeleteOne, FromQueryResult, Insert,
    ModelTrait, PrimaryKeyToColumn, PrimaryKeyTrait, QueryFilter, Related, RelationBuilder,
    RelationTrait, RelationType, Select, Update, UpdateMany, UpdateOne,
};
use sea_query::{Alias, Iden, IntoIden, IntoTableRef, IntoValueTuple, TableRef};
use std::fmt::Debug;
pub use strum::IntoEnumIterator as Iterable;

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

    /// Construct select statement to find one / all models excluding models that are being soft deleted
    ///
    /// - To select columns, join tables and group by expressions, see [`QuerySelect`](crate::query::QuerySelect)
    /// - To apply where conditions / filters, see [`QueryFilter`](crate::query::QueryFilter)
    /// - To apply order by expressions, see [`QueryOrder`](crate::query::QueryOrder)
    ///
    /// # Example (without soft delete)
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
    ///
    /// # Example (with soft delete)
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
    /// #             vendor::Model {
    /// #                 id: 1,
    /// #                 name: "Vendor A".to_owned(),
    /// #                 deleted_at: None,
    /// #             },
    /// #         ],
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::vendor};
    ///
    /// assert_eq!(
    ///     vendor::Entity::find().all(&db).await?,
    ///     vec![
    ///         vendor::Model {
    ///             id: 1,
    ///             name: "Vendor A".to_owned(),
    ///             deleted_at: None,
    ///         },
    ///     ]
    /// );
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![
    ///         Transaction::from_sql_and_values(
    ///             DbBackend::Postgres,
    ///             r#"SELECT "vendor"."id", "vendor"."name", "vendor"."deleted_at" FROM "vendor" WHERE "vendor"."deleted_at" IS NULL"#,
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

    /// Construct select statement to find one / all models that are being soft deleted
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
    /// #             vendor::Model {
    /// #                 id: 2,
    /// #                 name: "Vendor B".to_owned(),
    /// #                 deleted_at: Some("2022-06-10T16:24:00+00:00".parse().unwrap()),
    /// #             },
    /// #         ],
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::vendor};
    ///
    /// assert_eq!(
    ///     vendor::Entity::find_deleted().all(&db).await?,
    ///     vec![
    ///         vendor::Model {
    ///             id: 2,
    ///             name: "Vendor B".to_owned(),
    ///             deleted_at: Some("2022-06-10T16:24:00+00:00".parse().unwrap()),
    ///         },
    ///     ]
    /// );
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![
    ///         Transaction::from_sql_and_values(
    ///             DbBackend::Postgres,
    ///             r#"SELECT "vendor"."id", "vendor"."name", "vendor"."deleted_at" FROM "vendor" WHERE "vendor"."deleted_at" IS NOT NULL"#,
    ///             vec![]
    ///         ),
    ///     ]
    /// );
    /// #
    /// # Ok(())
    /// # }
    /// ```
    fn find_deleted() -> Select<Self> {
        Select::deleted()
    }

    /// Construct select statement to find one / all models including models that are being soft deleted
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
    /// #             vendor::Model {
    /// #                 id: 1,
    /// #                 name: "Vendor A".to_owned(),
    /// #                 deleted_at: None,
    /// #             },
    /// #             vendor::Model {
    /// #                 id: 2,
    /// #                 name: "Vendor B".to_owned(),
    /// #                 deleted_at: Some("2022-06-10T16:24:00+00:00".parse().unwrap()),
    /// #             },
    /// #         ],
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::vendor};
    ///
    /// assert_eq!(
    ///     vendor::Entity::find_with_deleted().all(&db).await?,
    ///     vec![
    ///         vendor::Model {
    ///             id: 1,
    ///             name: "Vendor A".to_owned(),
    ///             deleted_at: None,
    ///         },
    ///         vendor::Model {
    ///             id: 2,
    ///             name: "Vendor B".to_owned(),
    ///             deleted_at: Some("2022-06-10T16:24:00+00:00".parse().unwrap()),
    ///         },
    ///     ]
    /// );
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"SELECT "vendor"."id", "vendor"."name", "vendor"."deleted_at" FROM "vendor""#,
    ///         vec![]
    ///     ),]
    /// );
    /// #
    /// # Ok(())
    /// # }
    /// ```
    fn find_with_deleted() -> Select<Self> {
        Select::with_deleted()
    }

    /// Find a model by primary key excluding model that is being soft deleted
    ///
    /// # Example (without soft delete)
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
    ///
    /// Find by composite key
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
    ///
    /// # Example (with soft delete)
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
    /// #            vendor::Model {
    /// #                id: 2,
    /// #                name: "Vendor B".to_owned(),
    /// #                deleted_at: None,
    /// #            },
    /// #         ],
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::vendor};
    ///
    /// assert_eq!(
    ///     vendor::Entity::find_by_id(2).all(&db).await?,
    ///     vec![vendor::Model {
    ///         id: 2,
    ///         name: "Vendor B".to_owned(),
    ///         deleted_at: None,
    ///     }]
    /// );
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"SELECT "vendor"."id", "vendor"."name", "vendor"."deleted_at" FROM "vendor" WHERE "vendor"."deleted_at" IS NULL AND "vendor"."id" = $1"#,
    ///         vec![2i32.into()]
    ///     )]
    /// );
    /// #
    /// # Ok(())
    /// # }
    /// ```
    fn find_by_id(values: <Self::PrimaryKey as PrimaryKeyTrait>::ValueType) -> Select<Self> {
        filter_by_primary_key::<Self, _>(Self::find(), values)
    }

    /// Find a soft deleted model by primary key
    ///
    /// # Example (with soft delete)
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
    /// #            vendor::Model {
    /// #                id: 2,
    /// #                name: "Vendor B".to_owned(),
    /// #                deleted_at: Some("2022-06-10T16:24:00+00:00".parse().unwrap()),
    /// #            },
    /// #         ],
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::vendor};
    ///
    /// assert_eq!(
    ///     vendor::Entity::find_deleted_by_id(2).all(&db).await?,
    ///     vec![vendor::Model {
    ///         id: 2,
    ///         name: "Vendor B".to_owned(),
    ///         deleted_at: Some("2022-06-10T16:24:00+00:00".parse().unwrap()),
    ///     }]
    /// );
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"SELECT "vendor"."id", "vendor"."name", "vendor"."deleted_at" FROM "vendor" WHERE "vendor"."deleted_at" IS NOT NULL AND "vendor"."id" = $1"#,
    ///         vec![2i32.into()]
    ///     )]
    /// );
    /// #
    /// # Ok(())
    /// # }
    /// ```
    fn find_deleted_by_id(
        values: <Self::PrimaryKey as PrimaryKeyTrait>::ValueType,
    ) -> Select<Self> {
        filter_by_primary_key::<Self, _>(Self::find_deleted(), values)
    }

    /// Find a model by primary key including model that is being soft deleted
    ///
    /// # Example (without soft delete)
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
    ///     cake::Entity::find_with_deleted_by_id(11).all(&db).await?,
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
    ///
    /// Find by composite key
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
    ///     cake_filling::Entity::find_with_deleted_by_id((2, 3)).all(&db).await?,
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
    ///
    /// # Example (with soft delete)
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
    /// #            vendor::Model {
    /// #                id: 2,
    /// #                name: "Vendor B".to_owned(),
    /// #                deleted_at: None,
    /// #            },
    /// #         ],
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::vendor};
    ///
    /// assert_eq!(
    ///     vendor::Entity::find_with_deleted_by_id(2).all(&db).await?,
    ///     vec![vendor::Model {
    ///         id: 2,
    ///         name: "Vendor B".to_owned(),
    ///         deleted_at: None,
    ///     }]
    /// );
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"SELECT "vendor"."id", "vendor"."name", "vendor"."deleted_at" FROM "vendor" WHERE "vendor"."id" = $1"#,
    ///         vec![2i32.into()]
    ///     )]
    /// );
    /// #
    /// # Ok(())
    /// # }
    /// ```
    fn find_with_deleted_by_id(
        values: <Self::PrimaryKey as PrimaryKeyTrait>::ValueType,
    ) -> Select<Self> {
        filter_by_primary_key::<Self, _>(Self::find_with_deleted(), values)
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
    /// #     .append_query_results(vec![vec![maplit::btreemap! {
    /// #         "id" => Into::<Value>::into(15),
    /// #     }]])
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
    /// assert_eq!(dbg!(insert_result.last_insert_id), 15);
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"INSERT INTO "cake" ("name") VALUES ($1) RETURNING "id""#,
    ///         vec!["Apple Pie".into()]
    ///     )]
    /// );
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
    ///     )]
    /// );
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
    /// #     .append_query_results(vec![vec![maplit::btreemap! {
    /// #         "id" => Into::<Value>::into(28),
    /// #     }]])
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
    /// let insert_result = cake::Entity::insert_many(vec![apple, orange])
    ///     .exec(&db)
    ///     .await?;
    ///
    /// assert_eq!(insert_result.last_insert_id, 28);
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"INSERT INTO "cake" ("name") VALUES ($1), ($2) RETURNING "id""#,
    ///         vec!["Apple Pie".into(), "Orange Scone".into()]
    ///     )]
    /// );
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
    /// let insert_result = cake::Entity::insert_many(vec![apple, orange])
    ///     .exec(&db)
    ///     .await?;
    ///
    /// assert_eq!(insert_result.last_insert_id, 28);
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::MySql,
    ///         r#"INSERT INTO `cake` (`name`) VALUES (?), (?)"#,
    ///         vec!["Apple Pie".into(), "Orange Scone".into()]
    ///     )]
    /// );
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
    /// #     .append_query_results(vec![
    /// #         vec![fruit::Model {
    /// #             id: 1,
    /// #             name: "Orange".to_owned(),
    /// #             cake_id: None,
    /// #         }],
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
    ///     fruit::Model {
    ///         id: 1,
    ///         name: "Orange".to_owned(),
    ///         cake_id: None,
    ///     }
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
    /// #     .append_exec_results(vec![
    /// #         MockExecResult {
    /// #             last_insert_id: 0,
    /// #             rows_affected: 1,
    /// #         },
    /// #     ])
    /// #     .append_query_results(vec![
    /// #         vec![fruit::Model {
    /// #             id: 1,
    /// #             name: "Orange".to_owned(),
    /// #             cake_id: None,
    /// #         }],
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
    ///     fruit::Model {
    ///         id: 1,
    ///         name: "Orange".to_owned(),
    ///         cake_id: None,
    ///     }
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
    /// use sea_orm::{
    ///     entity::*,
    ///     query::*,
    ///     sea_query::{Expr, Value},
    ///     tests_cfg::fruit,
    /// };
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
    ///     )]
    /// );
    /// #
    /// # Ok(())
    /// # }
    /// ```
    fn update_many() -> UpdateMany<Self> {
        Update::many(Self::default())
    }

    /// Delete a model by:
    ///   - Marking the target row in the database as deleted if soft delete is enabled
    ///   - Otherwise, deleting the target row from the database
    ///
    /// To apply where conditions / filters, see [`QueryFilter`](crate::query::QueryFilter)
    ///
    /// # Example (without soft delete)
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
    ///         DbBackend::Postgres,
    ///         r#"DELETE FROM "fruit" WHERE "fruit"."id" = $1"#,
    ///         vec![3i32.into()]
    ///     )]
    /// );
    /// #
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Example (with soft delete)
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
    /// use sea_orm::{entity::*, query::*, tests_cfg::vendor};
    ///
    /// let vendor = vendor::ActiveModel {
    ///     id: Set(3),
    ///     ..Default::default()
    /// };
    ///
    /// let delete_result = vendor::Entity::delete(vendor).exec(&db).await?;
    ///
    /// assert_eq!(delete_result.rows_affected, 1);
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"UPDATE "vendor" SET "deleted_at" = CURRENT_TIMESTAMP WHERE "vendor"."id" = $1"#,
    ///         vec![3i32.into()]
    ///     )]
    /// );
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
    /// let delete_result = orange.delete_force(&db).await?;
    ///
    /// assert_eq!(delete_result.rows_affected, 1);
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"DELETE FROM "fruit" WHERE "fruit"."id" = $1"#,
    ///         vec![3i32.into()]
    ///     )]
    /// );
    /// #
    /// # Ok(())
    /// # }
    /// ```
    fn delete_force<A>(model: A) -> DeleteOne<A>
    where
        A: ActiveModelTrait<Entity = Self>,
    {
        Delete::one_force(model)
    }

    /// Delete many models by:
    ///   - Marking the target rows in the database as deleted if soft delete is enabled
    ///   - Otherwise, deleting the target rows from the database
    ///
    /// To apply where conditions / filters, see [`QueryFilter`](crate::query::QueryFilter)
    ///
    /// # Example (without soft delete)
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
    /// #     .append_query_results(vec![
    /// #         vec![fruit::Model {
    /// #             id: 15,
    /// #             name: "Apple Pie".to_owned(),
    /// #             cake_id: None,
    /// #         }],
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
    ///     )]
    /// );
    /// #
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Example (with soft delete)
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
    /// #     .append_query_results(vec![
    /// #         vec![vendor::Model {
    /// #             id: 15,
    /// #             name: "ABC Bakery".to_owned(),
    /// #             deleted_at: Some("2022-06-10T16:24:00+00:00".parse().unwrap()),
    /// #         }],
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::vendor};
    ///
    /// let delete_result = vendor::Entity::delete_many()
    ///     .filter(vendor::Column::Name.contains("Bakery"))
    ///     .exec(&db)
    ///     .await?;
    ///
    /// assert_eq!(delete_result.rows_affected, 5);
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"UPDATE "vendor" SET "deleted_at" = CURRENT_TIMESTAMP WHERE "vendor"."name" LIKE $1"#,
    ///         vec!["%Bakery%".into()]
    ///     )]
    /// );
    /// #
    /// # Ok(())
    /// # }
    /// ```
    fn delete_many() -> DeleteMany<Self> {
        Delete::many(Self::default())
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
    /// #     .append_query_results(vec![
    /// #         vec![fruit::Model {
    /// #             id: 15,
    /// #             name: "Apple Pie".to_owned(),
    /// #             cake_id: None,
    /// #         }],
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::fruit};
    ///
    /// let delete_result = fruit::Entity::delete_many_force()
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
    ///     )]
    /// );
    /// #
    /// # Ok(())
    /// # }
    /// ```
    fn delete_many_force() -> DeleteMany<Self> {
        Delete::many_force(Self::default())
    }

    /// Delete a model based on primary key by:
    ///   - Marking the target row in the database as deleted if soft delete is enabled
    ///   - Otherwise, deleting the target row from the database
    ///
    /// # Example (without soft delete)
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
    /// let delete_result = fruit::Entity::delete_by_id(1).exec(&db).await?;
    ///
    /// assert_eq!(delete_result.rows_affected, 1);
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"DELETE FROM "fruit" WHERE "fruit"."id" = $1"#,
    ///         vec![1i32.into()]
    ///     )]
    /// );
    /// #
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Delete by composite key
    ///
    /// ```
    /// # use sea_orm::{error::*, tests_cfg::*, *};
    /// #
    /// # #[smol_potat::main]
    /// # #[cfg(feature = "mock")]
    /// # pub async fn main() -> Result<(), DbErr> {
    ///
    /// # let db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_exec_results(vec![
    /// #         MockExecResult {
    /// #             last_insert_id: 0,
    /// #             rows_affected: 1,
    /// #         },
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake_filling};
    ///
    /// let delete_result = cake_filling::Entity::delete_by_id((2, 3)).exec(&db).await?;
    ///
    /// assert_eq!(delete_result.rows_affected, 1);
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"DELETE FROM "cake_filling" WHERE "cake_filling"."cake_id" = $1 AND "cake_filling"."filling_id" = $2"#,
    ///         vec![2i32.into(), 3i32.into()]
    ///     )]
    /// );
    /// #
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Example (with soft delete)
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
    /// use sea_orm::{entity::*, query::*, tests_cfg::vendor};
    ///
    /// let delete_result = vendor::Entity::delete_by_id(1).exec(&db).await?;
    ///
    /// assert_eq!(delete_result.rows_affected, 1);
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"UPDATE "vendor" SET "deleted_at" = CURRENT_TIMESTAMP WHERE "vendor"."id" = $1"#,
    ///         vec![1i32.into()]
    ///     )]
    /// );
    /// #
    /// # Ok(())
    /// # }
    /// ```
    fn delete_by_id(values: <Self::PrimaryKey as PrimaryKeyTrait>::ValueType) -> DeleteMany<Self> {
        filter_by_primary_key::<Self, _>(Self::delete_many(), values)
    }

    /// Delete a model based on primary key
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
    /// let delete_result = fruit::Entity::delete_by_id_force(1).exec(&db).await?;
    ///
    /// assert_eq!(delete_result.rows_affected, 1);
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"DELETE FROM "fruit" WHERE "fruit"."id" = $1"#,
    ///         vec![1i32.into()]
    ///     )]
    /// );
    /// #
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Delete by composite key
    ///
    /// ```
    /// # use sea_orm::{error::*, tests_cfg::*, *};
    /// #
    /// # #[smol_potat::main]
    /// # #[cfg(feature = "mock")]
    /// # pub async fn main() -> Result<(), DbErr> {
    ///
    /// # let db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_exec_results(vec![
    /// #         MockExecResult {
    /// #             last_insert_id: 0,
    /// #             rows_affected: 1,
    /// #         },
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake_filling};
    ///
    /// let delete_result = cake_filling::Entity::delete_by_id_force((2, 3)).exec(&db).await?;
    ///
    /// assert_eq!(delete_result.rows_affected, 1);
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"DELETE FROM "cake_filling" WHERE "cake_filling"."cake_id" = $1 AND "cake_filling"."filling_id" = $2"#,
    ///         vec![2i32.into(), 3i32.into()]
    ///     )]
    /// );
    /// #
    /// # Ok(())
    /// # }
    /// ```
    fn delete_by_id_force(
        values: <Self::PrimaryKey as PrimaryKeyTrait>::ValueType,
    ) -> DeleteMany<Self> {
        filter_by_primary_key::<Self, _>(Self::delete_many_force(), values)
    }
}

fn filter_by_primary_key<E, Q>(
    mut stmt: Q,
    values: <E::PrimaryKey as PrimaryKeyTrait>::ValueType,
) -> Q
where
    E: EntityTrait,
    Q: QueryFilter,
{
    let mut keys = E::PrimaryKey::iter();
    for v in values.into_value_tuple() {
        if let Some(key) = keys.next() {
            let col = key.into_column();
            stmt = stmt.filter(col.eq(v));
        } else {
            panic!("primary key arity mismatch");
        }
    }
    if keys.next().is_some() {
        panic!("primary key arity mismatch");
    }
    stmt
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_delete_by_id_1() {
        use crate::tests_cfg::cake;
        use crate::{entity::*, query::*, DbBackend};
        assert_eq!(
            cake::Entity::delete_by_id(1)
                .build(DbBackend::Sqlite)
                .to_string(),
            r#"DELETE FROM "cake" WHERE "cake"."id" = 1"#,
        );
    }

    #[test]
    fn test_delete_by_id_2() {
        use crate::tests_cfg::cake_filling_price;
        use crate::{entity::*, query::*, DbBackend};
        assert_eq!(
            cake_filling_price::Entity::delete_by_id_force((1, 2))
                .build(DbBackend::Sqlite)
                .to_string(),
            [
                r#"DELETE FROM "public"."cake_filling_price""#,
                r#"WHERE "cake_filling_price"."cake_id" = 1 AND "cake_filling_price"."filling_id" = 2"#,
            ].join(" ")
        );
        assert_eq!(
            cake_filling_price::Entity::delete_by_id((1, 2))
                .build(DbBackend::Sqlite)
                .to_string(),
            [
                r#"UPDATE "public"."cake_filling_price""#,
                r#"SET "deleted_at" = CURRENT_TIMESTAMP"#,
                r#"WHERE "cake_filling_price"."cake_id" = 1 AND "cake_filling_price"."filling_id" = 2"#,
            ].join(" ")
        );
    }

    #[test]
    #[cfg(feature = "macros")]
    fn entity_model_1() {
        use crate::entity::*;

        mod hello {
            use crate as sea_orm;
            use crate::entity::prelude::*;

            #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
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

            #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
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
