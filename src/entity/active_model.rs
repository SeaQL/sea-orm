use super::{ActiveValue, ActiveValue::*};
use crate::{
    ColumnTrait, ConnectionTrait, DeleteResult, EntityTrait, IdenStatic, Iterable, PrimaryKeyArity,
    PrimaryKeyToColumn, PrimaryKeyTrait, Related, RelationType, Value,
    error::*,
    query::{get_key_from_active_model, set_key_on_active_model},
};
use async_trait::async_trait;
use sea_query::ValueTuple;
use std::fmt::Debug;

/// `ActiveModel` is a type for constructing `INSERT` and `UPDATE` statements for a particular table.
///
/// Like [Model][ModelTrait], it represents a database record and each field represents a column.
///
/// But unlike [Model][ModelTrait], it also stores [additional state][ActiveValue] for every field,
/// and fields are not guaranteed to have a value.
///
/// This allows you to:
///
/// - omit columns from the query,
/// - know which columns have changed after editing a record.
#[async_trait]
pub trait ActiveModelTrait: Clone + Debug {
    /// The Entity this ActiveModel belongs to
    type Entity: EntityTrait;

    /// Get a mutable [ActiveValue] from an ActiveModel
    fn take(&mut self, c: <Self::Entity as EntityTrait>::Column) -> ActiveValue<Value>;

    /// Get a immutable [ActiveValue] from an ActiveModel
    fn get(&self, c: <Self::Entity as EntityTrait>::Column) -> ActiveValue<Value>;

    /// Set the Value of a ActiveModel field, panic if failed
    fn set(&mut self, c: <Self::Entity as EntityTrait>::Column, v: Value) {
        self.try_set(c, v)
            .unwrap_or_else(|e| panic!("Failed to set value for {:?}: {e:?}", c.as_column_ref()))
    }

    /// Set the Value of a ActiveModel field if value is different, panic if failed
    fn set_if_not_equals(&mut self, c: <Self::Entity as EntityTrait>::Column, v: Value);

    /// Set the Value of a ActiveModel field, return error if failed
    fn try_set(&mut self, c: <Self::Entity as EntityTrait>::Column, v: Value) -> Result<(), DbErr>;

    /// Set the state of an [ActiveValue] to the not set state
    fn not_set(&mut self, c: <Self::Entity as EntityTrait>::Column);

    /// Check the state of a [ActiveValue]
    fn is_not_set(&self, c: <Self::Entity as EntityTrait>::Column) -> bool;

    /// Create an ActiveModel with all fields to NotSet
    fn default() -> Self;

    /// Create an ActiveModel with all fields to Set(default_value) if Default is implemented, NotSet otherwise
    fn default_values() -> Self;

    /// Reset the value from [ActiveValue::Unchanged] to [ActiveValue::Set],
    /// leaving [ActiveValue::NotSet] untouched.
    fn reset(&mut self, c: <Self::Entity as EntityTrait>::Column);

    /// Reset all values from [ActiveValue::Unchanged] to [ActiveValue::Set],
    /// leaving [ActiveValue::NotSet] untouched.
    fn reset_all(mut self) -> Self {
        for col in <Self::Entity as EntityTrait>::Column::iter() {
            self.reset(col);
        }
        self
    }

    /// Get the primary key of the ActiveModel, only if it's fully specified.
    fn get_primary_key_value(&self) -> Option<ValueTuple> {
        let mut cols = <Self::Entity as EntityTrait>::PrimaryKey::iter();
        macro_rules! next {
            () => {
                self.get(cols.next()?.into_column()).into_value()?
            };
        }
        match <<<Self::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType as PrimaryKeyArity>::ARITY {
            1 => {
                let s1 = next!();
                Some(ValueTuple::One(s1))
            }
            2 => {
                let s1 = next!();
                let s2 = next!();
                Some(ValueTuple::Two(s1, s2))
            }
            3 => {
                let s1 = next!();
                let s2 = next!();
                let s3 = next!();
                Some(ValueTuple::Three(s1, s2, s3))
            }
            len => {
                let mut vec = Vec::with_capacity(len);
                for _ in 0..len {
                    let s = next!();
                    vec.push(s);
                }
                Some(ValueTuple::Many(vec))
            }
        }
    }

    /// Perform an `INSERT` operation on the ActiveModel
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
    /// #     .append_query_results([
    /// #         [cake::Model {
    /// #             id: 15,
    /// #             name: "Apple Pie".to_owned(),
    /// #         }],
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
    /// assert_eq!(
    ///     apple.insert(&db).await?,
    ///     cake::Model {
    ///         id: 15,
    ///         name: "Apple Pie".to_owned(),
    ///     }
    /// );
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     [Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"INSERT INTO "cake" ("name") VALUES ($1) RETURNING "id", "name""#,
    ///         ["Apple Pie".into()]
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
    /// #     .append_query_results([
    /// #         [cake::Model {
    /// #             id: 15,
    /// #             name: "Apple Pie".to_owned(),
    /// #         }],
    /// #     ])
    /// #     .append_exec_results([
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
    /// assert_eq!(
    ///     apple.insert(&db).await?,
    ///     cake::Model {
    ///         id: 15,
    ///         name: "Apple Pie".to_owned(),
    ///     }
    /// );
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     [
    ///         Transaction::from_sql_and_values(
    ///             DbBackend::MySql,
    ///             r#"INSERT INTO `cake` (`name`) VALUES (?)"#,
    ///             ["Apple Pie".into()]
    ///         ),
    ///         Transaction::from_sql_and_values(
    ///             DbBackend::MySql,
    ///             r#"SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`id` = ? LIMIT ?"#,
    ///             [15.into(), 1u64.into()]
    ///         )
    ///     ]
    /// );
    /// #
    /// # Ok(())
    /// # }
    /// ```
    async fn insert<'a, C>(self, db: &'a C) -> Result<<Self::Entity as EntityTrait>::Model, DbErr>
    where
        <Self::Entity as EntityTrait>::Model: IntoActiveModel<Self>,
        Self: ActiveModelBehavior + 'a,
        C: ConnectionTrait,
    {
        let am = ActiveModelBehavior::before_save(self, db, true).await?;
        let model = <Self::Entity as EntityTrait>::insert(am)
            .exec_with_returning(db)
            .await?;
        Self::after_save(model, db, true).await
    }

    /// Perform the `UPDATE` operation on an ActiveModel
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
    /// #     .append_query_results([
    /// #         [fruit::Model {
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
    ///     orange.update(&db).await?,
    ///     fruit::Model {
    ///         id: 1,
    ///         name: "Orange".to_owned(),
    ///         cake_id: None,
    ///     }
    /// );
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     [Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"UPDATE "fruit" SET "name" = $1 WHERE "fruit"."id" = $2 RETURNING "id", "name", "cake_id""#,
    ///         ["Orange".into(), 1i32.into()]
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
    /// #     .append_query_results([
    /// #         [fruit::Model {
    /// #             id: 1,
    /// #             name: "Orange".to_owned(),
    /// #             cake_id: None,
    /// #         }],
    /// #     ])
    /// #     .append_exec_results([
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
    ///     orange.update(&db).await?,
    ///     fruit::Model {
    ///         id: 1,
    ///         name: "Orange".to_owned(),
    ///         cake_id: None,
    ///     }
    /// );
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     [
    ///         Transaction::from_sql_and_values(
    ///             DbBackend::MySql,
    ///             r#"UPDATE `fruit` SET `name` = ? WHERE `fruit`.`id` = ?"#,
    ///             ["Orange".into(), 1i32.into()]
    ///         ),
    ///         Transaction::from_sql_and_values(
    ///             DbBackend::MySql,
    ///             r#"SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit` WHERE `fruit`.`id` = ? LIMIT ?"#,
    ///             [1i32.into(), 1u64.into()]
    ///         )]);
    /// #
    /// # Ok(())
    /// # }
    /// ```
    async fn update<'a, C>(self, db: &'a C) -> Result<<Self::Entity as EntityTrait>::Model, DbErr>
    where
        <Self::Entity as EntityTrait>::Model: IntoActiveModel<Self>,
        Self: ActiveModelBehavior + 'a,
        C: ConnectionTrait,
    {
        let am = ActiveModelBehavior::before_save(self, db, false).await?;
        let model: <Self::Entity as EntityTrait>::Model = Self::Entity::update(am).exec(db).await?;
        Self::after_save(model, db, false).await
    }

    /// Insert the model if primary key is `NotSet`, update otherwise.
    /// Only works if the entity has auto increment primary key.
    async fn save<'a, C>(self, db: &'a C) -> Result<Self, DbErr>
    where
        <Self::Entity as EntityTrait>::Model: IntoActiveModel<Self>,
        Self: ActiveModelBehavior + 'a,
        C: ConnectionTrait,
    {
        let res = if !self.is_update() {
            self.insert(db).await
        } else {
            self.update(db).await
        }?;
        Ok(res.into_active_model())
    }

    /// Returns true if the primary key is fully-specified
    #[doc(hidden)]
    fn is_update(&self) -> bool {
        let mut is_update = true;
        for key in <Self::Entity as EntityTrait>::PrimaryKey::iter() {
            let col = key.into_column();
            if self.is_not_set(col) {
                is_update = false;
                break;
            }
        }
        is_update
    }

    /// Delete an active model by its primary key
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
    /// #     .append_exec_results([
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
    /// let delete_result = orange.delete(&db).await?;
    ///
    /// assert_eq!(delete_result.rows_affected, 1);
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     [Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"DELETE FROM "fruit" WHERE "fruit"."id" = $1"#,
    ///         [3i32.into()]
    ///     )]
    /// );
    /// #
    /// # Ok(())
    /// # }
    /// ```
    async fn delete<'a, C>(self, db: &'a C) -> Result<DeleteResult, DbErr>
    where
        Self: ActiveModelBehavior + 'a,
        C: ConnectionTrait,
    {
        let am = ActiveModelBehavior::before_delete(self, db).await?;
        let am_clone = am.clone();
        let delete_res = Self::Entity::delete(am).exec(db).await?;
        ActiveModelBehavior::after_delete(am_clone, db).await?;
        Ok(delete_res)
    }

    /// Set the corresponding attributes in the ActiveModel from a JSON value
    ///
    /// Note that this method will not alter the primary key values in ActiveModel.
    #[cfg(feature = "with-json")]
    fn set_from_json(&mut self, json: serde_json::Value) -> Result<(), DbErr>
    where
        Self: crate::TryIntoModel<<Self::Entity as EntityTrait>::Model>,
        <<Self as ActiveModelTrait>::Entity as EntityTrait>::Model: IntoActiveModel<Self>,
        for<'de> <<Self as ActiveModelTrait>::Entity as EntityTrait>::Model:
            serde::de::Deserialize<'de> + serde::Serialize,
    {
        use crate::Iterable;

        // Backup primary key values
        let primary_key_values: Vec<(<Self::Entity as EntityTrait>::Column, ActiveValue<Value>)> =
            <<Self::Entity as EntityTrait>::PrimaryKey>::iter()
                .map(|pk| (pk.into_column(), self.take(pk.into_column())))
                .collect();

        // Replace all values in ActiveModel
        *self = Self::from_json(json)?;

        // Restore primary key values
        for (col, active_value) in primary_key_values {
            match active_value {
                ActiveValue::Unchanged(v) | ActiveValue::Set(v) => self.set(col, v),
                NotSet => self.not_set(col),
            }
        }

        Ok(())
    }

    /// Create ActiveModel from a JSON value
    #[cfg(feature = "with-json")]
    fn from_json(mut json: serde_json::Value) -> Result<Self, DbErr>
    where
        Self: crate::TryIntoModel<<Self::Entity as EntityTrait>::Model>,
        <<Self as ActiveModelTrait>::Entity as EntityTrait>::Model: IntoActiveModel<Self>,
        for<'de> <<Self as ActiveModelTrait>::Entity as EntityTrait>::Model:
            serde::de::Deserialize<'de> + serde::Serialize,
    {
        use crate::{IdenStatic, Iterable};

        let serde_json::Value::Object(obj) = &json else {
            return Err(DbErr::Json(format!(
                "invalid type: expected JSON object for {}",
                <<Self as ActiveModelTrait>::Entity as IdenStatic>::as_str(&Default::default())
            )));
        };

        // Mark down which attribute exists in the JSON object
        let mut json_keys: Vec<(<Self::Entity as EntityTrait>::Column, bool)> = Vec::new();

        for col in <<Self::Entity as EntityTrait>::Column>::iter() {
            let key = col.as_str();
            let has_key = obj.contains_key(key);
            json_keys.push((col, has_key));
        }

        // Create dummy model with dummy values
        let dummy_model = Self::default_values();
        if let Ok(dummy_model) = dummy_model.try_into_model() {
            if let Ok(mut dummy_json) = serde_json::to_value(&dummy_model) {
                let serde_json::Value::Object(merged) = &mut dummy_json else {
                    unreachable!();
                };
                let serde_json::Value::Object(obj) = json else {
                    unreachable!();
                };
                // overwrite dummy values with input values
                for (key, value) in obj {
                    merged.insert(key, value);
                }
                json = dummy_json;
            }
        }

        // Convert JSON object into ActiveModel via Model
        let model: <Self::Entity as EntityTrait>::Model =
            serde_json::from_value(json).map_err(json_err)?;
        let mut am = model.into_active_model();

        // Transform attribute that exists in JSON object into ActiveValue::Set, otherwise ActiveValue::NotSet
        for (col, json_key_exists) in json_keys {
            match (json_key_exists, am.get(col)) {
                (true, ActiveValue::Set(value) | ActiveValue::Unchanged(value)) => {
                    am.set(col, value);
                }
                _ => {
                    am.not_set(col);
                }
            }
        }

        Ok(am)
    }

    /// Return `true` if any attribute of `ActiveModel` is `Set`
    fn is_changed(&self) -> bool {
        <Self::Entity as EntityTrait>::Column::iter()
            .any(|col| matches!(self.get(col), ActiveValue::Set(_)))
    }

    #[doc(hidden)]
    /// Used by ActiveModelEx. Set the key to parent's key value for a belongs to relation.
    fn set_parent_key<R, AM>(&mut self, model: &AM) -> Result<(), DbErr>
    where
        R: EntityTrait,
        AM: ActiveModelTrait<Entity = R>,
        Self::Entity: Related<R>,
    {
        let rel_def = Self::Entity::to();

        if rel_def.rel_type != RelationType::HasOne {
            return Err(DbErr::Type(format!(
                "Relation from {} to {} is not HasOne",
                <Self::Entity as Default>::default().as_str(),
                <R as Default>::default().as_str()
            )));
        }

        if rel_def.is_owner {
            return Err(DbErr::Type(format!(
                "Relation from {} to {} is not belongs_to",
                <Self::Entity as Default>::default().as_str(),
                <R as Default>::default().as_str()
            )));
        }

        let values = get_key_from_active_model(&rel_def.to_col, model)?;

        set_key_on_active_model(&rel_def.from_col, values, self)?;

        Ok(())
    }

    #[doc(hidden)]
    fn get_parent_key<R, AM>(&self, _: &AM) -> Result<ValueTuple, DbErr>
    where
        R: EntityTrait,
        AM: ActiveModelTrait<Entity = R>,
        Self::Entity: Related<R>,
    {
        let rel_def = Self::Entity::to();

        if rel_def.rel_type != RelationType::HasOne {
            return Err(DbErr::Type(format!(
                "Relation from {} to {} is not HasOne",
                <Self::Entity as Default>::default().as_str(),
                <R as Default>::default().as_str()
            )));
        }

        if rel_def.is_owner {
            return Err(DbErr::Type(format!(
                "Relation from {} to {} is not belongs_to",
                <Self::Entity as Default>::default().as_str(),
                <R as Default>::default().as_str()
            )));
        }

        get_key_from_active_model(&rel_def.from_col, self)
    }

    #[doc(hidden)]
    fn find_related_of<AM>(&self, _: &[AM]) -> crate::query::Select<AM::Entity>
    where
        AM: ActiveModelTrait,
        Self::Entity: Related<AM::Entity>,
    {
        use crate::query::QueryFilter;

        Self::Entity::find_related().belongs_to_active_model(self)
    }
}

/// A Trait for overriding the ActiveModel behavior
///
/// ### Example
/// ```ignore
/// use sea_orm::entity::prelude::*;
///
///  // Use [DeriveEntity] to derive the EntityTrait automatically
/// #[derive(Copy, Clone, Default, Debug, DeriveEntity)]
/// pub struct Entity;
///
/// /// The [EntityName] describes the name of a table
/// impl EntityName for Entity {
///     fn table_name(&self) -> &'static str {
///         "cake"
///     }
/// }
///
/// // Derive the ActiveModel
/// #[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel)]
/// pub struct Model {
///     pub id: i32,
///     pub name: String,
/// }
///
/// impl ActiveModelBehavior for ActiveModel {}
/// ```
/// See module level docs [crate::entity] for a full example
#[allow(unused_variables)]
#[async_trait]
pub trait ActiveModelBehavior: ActiveModelTrait {
    /// Create a new ActiveModel with default values. This is also called by `Default::default()`.
    ///
    /// You can override it like the following:
    ///
    /// ```ignore
    /// fn new() -> Self {
    ///     Self {
    ///         status: Set(Status::New),
    ///         ..ActiveModelTrait::default()
    ///     }
    /// }
    /// ```
    fn new() -> Self {
        <Self as ActiveModelTrait>::default()
    }

    /// Will be called before `ActiveModel::insert`, `ActiveModel::update`, and `ActiveModel::save`
    async fn before_save<C>(self, db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        Ok(self)
    }

    /// Will be called after `ActiveModel::insert`, `ActiveModel::update`, and `ActiveModel::save`
    async fn after_save<C>(
        model: <Self::Entity as EntityTrait>::Model,
        db: &C,
        insert: bool,
    ) -> Result<<Self::Entity as EntityTrait>::Model, DbErr>
    where
        C: ConnectionTrait,
    {
        Ok(model)
    }

    /// Will be called before `ActiveModel::delete`
    async fn before_delete<C>(self, db: &C) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        Ok(self)
    }

    /// Will be called after `ActiveModel::delete`
    async fn after_delete<C>(self, db: &C) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        Ok(self)
    }
}

/// A Trait for any type that can be converted into an ActiveModel
pub trait IntoActiveModel<A>
where
    A: ActiveModelTrait,
{
    /// Method to call to perform the conversion
    fn into_active_model(self) -> A;
}

impl<A> IntoActiveModel<A> for A
where
    A: ActiveModelTrait,
{
    fn into_active_model(self) -> A {
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::{DbErr, entity::*, tests_cfg::*};
    use pretty_assertions::assert_eq;

    #[cfg(feature = "with-json")]
    use serde_json::json;

    #[test]
    #[cfg(feature = "macros")]
    fn test_derive_into_active_model_1() {
        mod my_fruit {
            pub use super::fruit::*;
            use crate as sea_orm;
            use crate::entity::prelude::*;

            #[derive(DeriveIntoActiveModel)]
            pub struct NewFruit {
                // id is omitted
                pub name: String,
                // it is required as opposed to optional in Model
                pub cake_id: i32,
            }
        }

        assert_eq!(
            my_fruit::NewFruit {
                name: "Apple".to_owned(),
                cake_id: 1,
            }
            .into_active_model(),
            fruit::ActiveModel {
                id: NotSet,
                name: Set("Apple".to_owned()),
                cake_id: Set(Some(1)),
            }
        );
    }

    #[test]
    #[cfg(feature = "macros")]
    fn test_derive_into_active_model_2() {
        use crate as sea_orm;
        use crate::entity::prelude::*;

        #[derive(DeriveIntoActiveModel)]
        #[sea_orm(active_model = "fruit::ActiveModel")]
        struct FruitName {
            name: String,
        }

        assert_eq!(
            FruitName {
                name: "Apple Pie".to_owned(),
            }
            .into_active_model(),
            fruit::ActiveModel {
                id: NotSet,
                name: Set("Apple Pie".to_owned()),
                cake_id: NotSet,
            }
        );

        #[derive(DeriveIntoActiveModel)]
        #[sea_orm(active_model = "<fruit::Entity as EntityTrait>::ActiveModel")]
        struct FruitCake {
            cake_id: Option<Option<i32>>,
        }

        assert_eq!(
            FruitCake {
                cake_id: Some(Some(1)),
            }
            .into_active_model(),
            fruit::ActiveModel {
                id: NotSet,
                name: NotSet,
                cake_id: Set(Some(1)),
            }
        );

        assert_eq!(
            FruitCake {
                cake_id: Some(None),
            }
            .into_active_model(),
            fruit::ActiveModel {
                id: NotSet,
                name: NotSet,
                cake_id: Set(None),
            }
        );

        assert_eq!(
            FruitCake { cake_id: None }.into_active_model(),
            fruit::ActiveModel {
                id: NotSet,
                name: NotSet,
                cake_id: NotSet,
            }
        );
    }

    #[test]
    #[cfg(feature = "macros")]
    fn test_derive_try_into_model_1() {
        mod my_fruit {
            use crate as sea_orm;
            use crate::entity::prelude::*;

            #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
            #[sea_orm(table_name = "fruit")]
            pub struct Model {
                #[sea_orm(primary_key)]
                pub id: i32,
                pub name: String,
                pub cake_id: Option<i32>,
            }

            #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
            pub enum Relation {}

            impl ActiveModelBehavior for ActiveModel {}
        }
        assert_eq!(
            my_fruit::ActiveModel {
                id: Set(1),
                name: Set("Pineapple".to_owned()),
                cake_id: Set(None),
            }
            .try_into_model()
            .unwrap(),
            my_fruit::Model {
                id: 1,
                name: "Pineapple".to_owned(),
                cake_id: None,
            }
        );

        assert_eq!(
            my_fruit::ActiveModel {
                id: Set(2),
                name: Set("Apple".to_owned()),
                cake_id: Set(Some(1)),
            }
            .try_into_model()
            .unwrap(),
            my_fruit::Model {
                id: 2,
                name: "Apple".to_owned(),
                cake_id: Some(1),
            }
        );

        assert_eq!(
            my_fruit::ActiveModel {
                id: Set(1),
                name: NotSet,
                cake_id: Set(None),
            }
            .try_into_model(),
            Err(DbErr::AttrNotSet(String::from("name")))
        );

        assert_eq!(
            my_fruit::ActiveModel {
                id: Set(1),
                name: Set("Pineapple".to_owned()),
                cake_id: NotSet,
            }
            .try_into_model(),
            Err(DbErr::AttrNotSet(String::from("cake_id")))
        );
    }

    #[test]
    #[cfg(feature = "macros")]
    fn test_derive_try_into_model_2() {
        mod my_fruit {
            use crate as sea_orm;
            use crate::entity::prelude::*;

            #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
            #[sea_orm(table_name = "fruit")]
            pub struct Model {
                #[sea_orm(primary_key)]
                pub id: i32,
                pub name: String,
                #[sea_orm(ignore)]
                pub cake_id: Option<i32>,
            }

            #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
            pub enum Relation {}

            impl ActiveModelBehavior for ActiveModel {}
        }
        assert_eq!(
            my_fruit::ActiveModel {
                id: Set(1),
                name: Set("Pineapple".to_owned()),
            }
            .try_into_model()
            .unwrap(),
            my_fruit::Model {
                id: 1,
                name: "Pineapple".to_owned(),
                cake_id: None,
            }
        );
    }

    #[test]
    #[cfg(feature = "macros")]
    fn test_derive_try_into_model_3() {
        mod my_fruit {
            use crate as sea_orm;
            use crate::entity::prelude::*;

            #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
            #[sea_orm(table_name = "fruit")]
            pub struct Model {
                #[sea_orm(primary_key)]
                pub id: i32,
                #[sea_orm(ignore)]
                pub name: String,
                pub cake_id: Option<i32>,
            }

            #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
            pub enum Relation {}

            impl ActiveModelBehavior for ActiveModel {}
        }
        assert_eq!(
            my_fruit::ActiveModel {
                id: Set(1),
                cake_id: Set(Some(1)),
            }
            .try_into_model()
            .unwrap(),
            my_fruit::Model {
                id: 1,
                name: "".to_owned(),
                cake_id: Some(1),
            }
        );
    }

    #[test]
    #[cfg(feature = "with-json")]
    fn test_active_model_set_from_json_1() {
        assert_eq!(
            cake::ActiveModel::from_json(json!({
                "id": 1,
                "name": "Apple Pie",
            }))
            .unwrap(),
            cake::ActiveModel {
                id: Set(1),
                name: Set("Apple Pie".to_owned()),
            }
        );

        assert_eq!(
            cake::ActiveModel::from_json(json!({
                "id": 1,
            }))
            .unwrap(),
            cake::ActiveModel {
                id: Set(1),
                name: NotSet,
            }
        );

        assert_eq!(
            cake::ActiveModel::from_json(json!({
                "name": "Apple Pie",
            }))
            .unwrap(),
            cake::ActiveModel {
                id: NotSet,
                name: Set("Apple Pie".to_owned()),
            }
        );

        let mut cake: cake::ActiveModel = Default::default();
        cake.set_from_json(json!({
            "name": "Apple Pie",
        }))
        .unwrap();
        assert_eq!(
            cake,
            cake::ActiveModel {
                id: NotSet,
                name: Set("Apple Pie".to_owned()),
            }
        );
    }

    #[test]
    #[cfg(feature = "with-json")]
    fn test_active_model_set_from_json_2() -> Result<(), DbErr> {
        let mut fruit: fruit::ActiveModel = Default::default();

        fruit.set_from_json(json!({
            "name": "Apple",
        }))?;
        assert_eq!(
            fruit,
            fruit::ActiveModel {
                id: ActiveValue::NotSet,
                name: ActiveValue::Set("Apple".to_owned()),
                cake_id: ActiveValue::NotSet,
            }
        );

        assert_eq!(
            fruit::ActiveModel::from_json(json!({
                "name": "Apple",
            }))?,
            fruit::ActiveModel {
                id: ActiveValue::NotSet,
                name: ActiveValue::Set("Apple".to_owned()),
                cake_id: ActiveValue::NotSet,
            }
        );

        fruit.set_from_json(json!({
            "name": "Apple",
            "cake_id": null,
        }))?;
        assert_eq!(
            fruit,
            fruit::ActiveModel {
                id: ActiveValue::NotSet,
                name: ActiveValue::Set("Apple".to_owned()),
                cake_id: ActiveValue::Set(None),
            }
        );

        fruit.set_from_json(json!({
            "id": null,
            "name": "Apple",
            "cake_id": 1,
        }))?;
        assert_eq!(
            fruit,
            fruit::ActiveModel {
                id: ActiveValue::NotSet,
                name: ActiveValue::Set("Apple".to_owned()),
                cake_id: ActiveValue::Set(Some(1)),
            }
        );

        fruit.set_from_json(json!({
            "id": 2,
            "name": "Apple",
            "cake_id": 1,
        }))?;
        assert_eq!(
            fruit,
            fruit::ActiveModel {
                id: ActiveValue::NotSet,
                name: ActiveValue::Set("Apple".to_owned()),
                cake_id: ActiveValue::Set(Some(1)),
            }
        );

        let mut fruit = fruit::ActiveModel {
            id: ActiveValue::Set(1),
            name: ActiveValue::NotSet,
            cake_id: ActiveValue::NotSet,
        };
        fruit.set_from_json(json!({
            "id": 8,
            "name": "Apple",
            "cake_id": 1,
        }))?;
        assert_eq!(
            fruit,
            fruit::ActiveModel {
                id: ActiveValue::Set(1),
                name: ActiveValue::Set("Apple".to_owned()),
                cake_id: ActiveValue::Set(Some(1)),
            }
        );

        Ok(())
    }

    #[smol_potat::test]
    #[cfg(feature = "with-json")]
    async fn test_active_model_set_from_json_3() -> Result<(), DbErr> {
        use crate::*;

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_exec_results([
                MockExecResult {
                    last_insert_id: 1,
                    rows_affected: 1,
                },
                MockExecResult {
                    last_insert_id: 1,
                    rows_affected: 1,
                },
            ])
            .append_query_results([
                [fruit::Model {
                    id: 1,
                    name: "Apple".to_owned(),
                    cake_id: None,
                }],
                [fruit::Model {
                    id: 2,
                    name: "Orange".to_owned(),
                    cake_id: Some(1),
                }],
            ])
            .into_connection();

        let mut fruit: fruit::ActiveModel = Default::default();
        fruit.set_from_json(json!({
            "name": "Apple",
        }))?;
        fruit.save(&db).await?;

        let mut fruit = fruit::ActiveModel {
            id: Set(2),
            ..Default::default()
        };
        fruit.set_from_json(json!({
            "id": 9,
            "name": "Orange",
            "cake_id": 1,
        }))?;
        fruit.save(&db).await?;

        assert_eq!(
            db.into_transaction_log(),
            [
                Transaction::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"INSERT INTO "fruit" ("name") VALUES ($1) RETURNING "id", "name", "cake_id""#,
                    ["Apple".into()],
                ),
                Transaction::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"UPDATE "fruit" SET "name" = $1, "cake_id" = $2 WHERE "fruit"."id" = $3 RETURNING "id", "name", "cake_id""#,
                    ["Orange".into(), 1i32.into(), 2i32.into()],
                ),
            ]
        );

        Ok(())
    }

    #[test]
    fn test_active_model_is_changed() {
        let mut fruit: fruit::ActiveModel = Default::default();
        assert!(!fruit.is_changed());

        fruit.set(fruit::Column::Name, "apple".into());
        assert!(fruit.is_changed());

        let mut fruit = fruit::Model {
            id: 1,
            name: "".into(),
            cake_id: None,
        };
        fruit.set("name".parse().unwrap(), "orange".into());
        assert_eq!(fruit.name, "orange");
    }

    #[test]
    fn test_reset_1() {
        assert_eq!(
            fruit::Model {
                id: 1,
                name: "Apple".into(),
                cake_id: None,
            }
            .into_active_model(),
            fruit::ActiveModel {
                id: Unchanged(1),
                name: Unchanged("Apple".into()),
                cake_id: Unchanged(None)
            },
        );

        assert_eq!(
            fruit::Model {
                id: 1,
                name: "Apple".into(),
                cake_id: None,
            }
            .into_active_model()
            .reset_all(),
            fruit::ActiveModel {
                id: Set(1),
                name: Set("Apple".into()),
                cake_id: Set(None)
            },
        );

        assert_eq!(
            fruit::Model {
                id: 1,
                name: "Apple".into(),
                cake_id: Some(2),
            }
            .into_active_model(),
            fruit::ActiveModel {
                id: Unchanged(1),
                name: Unchanged("Apple".into()),
                cake_id: Unchanged(Some(2)),
            },
        );

        assert_eq!(
            fruit::Model {
                id: 1,
                name: "Apple".into(),
                cake_id: Some(2),
            }
            .into_active_model()
            .reset_all(),
            fruit::ActiveModel {
                id: Set(1),
                name: Set("Apple".into()),
                cake_id: Set(Some(2)),
            },
        );
    }

    #[smol_potat::test]
    async fn test_reset_2() -> Result<(), DbErr> {
        use crate::*;

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_exec_results(vec![
                MockExecResult {
                    last_insert_id: 1,
                    rows_affected: 1,
                },
                MockExecResult {
                    last_insert_id: 1,
                    rows_affected: 1,
                },
            ])
            .append_query_results(vec![
                vec![fruit::Model {
                    id: 1,
                    name: "Apple".to_owned(),
                    cake_id: None,
                }],
                vec![fruit::Model {
                    id: 1,
                    name: "Apple".to_owned(),
                    cake_id: None,
                }],
            ])
            .into_connection();

        fruit::Model {
            id: 1,
            name: "Apple".into(),
            cake_id: None,
        }
        .into_active_model()
        .update(&db)
        .await?;

        fruit::Model {
            id: 1,
            name: "Apple".into(),
            cake_id: None,
        }
        .into_active_model()
        .reset_all()
        .update(&db)
        .await?;

        assert_eq!(
            db.into_transaction_log(),
            vec![
                Transaction::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"SELECT "fruit"."id", "fruit"."name", "fruit"."cake_id" FROM "fruit" WHERE "fruit"."id" = $1 LIMIT $2"#,
                    vec![1i32.into(), 1u64.into()],
                ),
                Transaction::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"UPDATE "fruit" SET "name" = $1, "cake_id" = $2 WHERE "fruit"."id" = $3 RETURNING "id", "name", "cake_id""#,
                    vec!["Apple".into(), Option::<i32>::None.into(), 1i32.into()],
                ),
            ]
        );

        Ok(())
    }

    #[test]
    fn test_active_model_default_values() {
        assert_eq!(
            fruit::ActiveModel::default_values(),
            fruit::ActiveModel {
                id: Set(0),
                name: Set("".into()),
                cake_id: Set(None),
            },
        );

        assert_eq!(
            lunch_set::ActiveModel::default_values(),
            lunch_set::ActiveModel {
                id: Set(0),
                name: Set("".into()),
                tea: NotSet,
            },
        );
    }

    #[test]
    fn test_active_model_set_parent_key() {
        let mut fruit = fruit::Model {
            id: 2,
            name: "F".into(),
            cake_id: None,
        }
        .into_active_model();

        let cake = cake::Model {
            id: 4,
            name: "C".into(),
        }
        .into_active_model();

        fruit.set_parent_key(&cake).unwrap();

        assert_eq!(
            fruit,
            fruit::ActiveModel {
                id: Unchanged(2),
                name: Unchanged("F".into()),
                cake_id: Set(Some(4)),
            }
        );

        let mut cake_filling = cake_filling::ActiveModel::new();

        cake_filling.set_parent_key(&cake).unwrap();

        assert_eq!(
            cake_filling,
            cake_filling::ActiveModel {
                cake_id: Set(4),
                filling_id: NotSet,
            }
        );
    }
}
