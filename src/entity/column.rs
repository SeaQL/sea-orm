use crate::{
    ColumnDef, ColumnType, DbBackend, EntityName, Iden, IdenStatic, IntoSimpleExpr, Iterable,
};
use sea_query::{
    Alias, BinOper, DynIden, Expr, ExprTrait, IntoIden, IntoLikeExpr, SeaRc, SelectStatement,
    SimpleExpr, Value,
};
use std::{borrow::Cow, str::FromStr};

macro_rules! bind_oper {
    ( $op: ident, $bin_op: ident ) => {
        #[allow(missing_docs)]
        fn $op<V>(&self, v: V) -> SimpleExpr
        where
            V: Into<Value>,
        {
            let expr = self.save_as(Expr::val(v));
            Expr::col((self.entity_name(), *self)).binary(BinOper::$bin_op, expr)
        }
    };
}

macro_rules! bind_func_no_params {
    ( $func: ident ) => {
        /// See also SeaQuery's method with same name.
        fn $func(&self) -> SimpleExpr {
            Expr::col((self.entity_name(), *self)).$func()
        }
    };
}

macro_rules! bind_vec_func {
    ( $func: ident ) => {
        #[allow(missing_docs)]
        #[allow(clippy::wrong_self_convention)]
        fn $func<V, I>(&self, v: I) -> SimpleExpr
        where
            V: Into<Value>,
            I: IntoIterator<Item = V>,
        {
            let v_with_enum_cast = v.into_iter().map(|v| self.save_as(Expr::val(v)));
            Expr::col((self.entity_name(), *self)).$func(v_with_enum_cast)
        }
    };
}

macro_rules! bind_subquery_func {
    ( $func: ident ) => {
        #[allow(clippy::wrong_self_convention)]
        #[allow(missing_docs)]
        fn $func(&self, s: SelectStatement) -> SimpleExpr {
            Expr::col((self.entity_name(), *self)).$func(s)
        }
    };
}

// LINT: when the operand value does not match column type
/// API for working with a `Column`. Mostly a wrapper of the identically named methods in [`sea_query::Expr`]
pub trait ColumnTrait: IdenStatic + Iterable + FromStr {
    #[allow(missing_docs)]
    type EntityName: EntityName;

    /// Define a column for an Entity
    fn def(&self) -> ColumnDef;

    /// Get the enum name of the column type
    fn enum_type_name(&self) -> Option<&'static str> {
        None
    }

    /// Get the name of the entity the column belongs to
    fn entity_name(&self) -> DynIden {
        SeaRc::new(Self::EntityName::default())
    }

    /// get the name of the entity the column belongs to
    fn as_column_ref(&self) -> (DynIden, DynIden) {
        (self.entity_name(), SeaRc::new(*self))
    }

    bind_oper!(eq, Equal);
    bind_oper!(ne, NotEqual);
    bind_oper!(gt, GreaterThan);
    bind_oper!(gte, GreaterThanOrEqual);
    bind_oper!(lt, SmallerThan);
    bind_oper!(lte, SmallerThanOrEqual);

    /// ```
    /// use sea_orm::{DbBackend, entity::*, query::*, tests_cfg::cake};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .filter(cake::Column::Id.between(2, 3))
    ///         .build(DbBackend::MySql)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`id` BETWEEN 2 AND 3"
    /// );
    /// ```
    fn between<V>(&self, a: V, b: V) -> SimpleExpr
    where
        V: Into<Value>,
    {
        Expr::col((self.entity_name(), *self)).between(a, b)
    }

    /// ```
    /// use sea_orm::{DbBackend, entity::*, query::*, tests_cfg::cake};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .filter(cake::Column::Id.not_between(2, 3))
    ///         .build(DbBackend::MySql)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`id` NOT BETWEEN 2 AND 3"
    /// );
    /// ```
    fn not_between<V>(&self, a: V, b: V) -> SimpleExpr
    where
        V: Into<Value>,
    {
        Expr::col((self.entity_name(), *self)).not_between(a, b)
    }

    /// ```
    /// use sea_orm::{DbBackend, entity::*, query::*, tests_cfg::cake};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .filter(cake::Column::Name.like("cheese"))
    ///         .build(DbBackend::MySql)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`name` LIKE 'cheese'"
    /// );
    /// ```
    fn like<T>(&self, s: T) -> SimpleExpr
    where
        T: IntoLikeExpr,
    {
        Expr::col((self.entity_name(), *self)).like(s)
    }

    /// ```
    /// use sea_orm::{DbBackend, entity::*, query::*, tests_cfg::cake};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .filter(cake::Column::Name.not_like("cheese"))
    ///         .build(DbBackend::MySql)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`name` NOT LIKE 'cheese'"
    /// );
    /// ```
    fn not_like<T>(&self, s: T) -> SimpleExpr
    where
        T: IntoLikeExpr,
    {
        Expr::col((self.entity_name(), *self)).not_like(s)
    }

    /// This is a simplified shorthand for a more general `like` method.
    /// Use `like` if you need something more complex, like specifying an escape character.
    ///
    /// ## Examples
    ///
    /// ```
    /// use sea_orm::{DbBackend, entity::*, query::*, tests_cfg::cake};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .filter(cake::Column::Name.starts_with("cheese"))
    ///         .build(DbBackend::MySql)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`name` LIKE 'cheese%'"
    /// );
    /// ```
    fn starts_with<T>(&self, s: T) -> SimpleExpr
    where
        T: Into<String>,
    {
        let pattern = format!("{}%", s.into());
        Expr::col((self.entity_name(), *self)).like(pattern)
    }

    /// This is a simplified shorthand for a more general `like` method.
    /// Use `like` if you need something more complex, like specifying an escape character.
    ///
    /// ## Examples
    ///
    /// ```
    /// use sea_orm::{DbBackend, entity::*, query::*, tests_cfg::cake};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .filter(cake::Column::Name.ends_with("cheese"))
    ///         .build(DbBackend::MySql)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`name` LIKE '%cheese'"
    /// );
    /// ```
    fn ends_with<T>(&self, s: T) -> SimpleExpr
    where
        T: Into<String>,
    {
        let pattern = format!("%{}", s.into());
        Expr::col((self.entity_name(), *self)).like(pattern)
    }

    /// This is a simplified shorthand for a more general `like` method.
    /// Use `like` if you need something more complex, like specifying an escape character.
    ///
    /// ## Examples
    ///
    /// ```
    /// use sea_orm::{DbBackend, entity::*, query::*, tests_cfg::cake};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .filter(cake::Column::Name.contains("cheese"))
    ///         .build(DbBackend::MySql)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`name` LIKE '%cheese%'"
    /// );
    /// ```
    fn contains<T>(&self, s: T) -> SimpleExpr
    where
        T: Into<String>,
    {
        let pattern = format!("%{}%", s.into());
        Expr::col((self.entity_name(), *self)).like(pattern)
    }

    bind_func_no_params!(max);
    bind_func_no_params!(min);
    bind_func_no_params!(sum);
    bind_func_no_params!(count);
    bind_func_no_params!(is_null);
    bind_func_no_params!(is_not_null);

    /// Perform an operation if the column is null
    fn if_null<V>(&self, v: V) -> SimpleExpr
    where
        V: Into<Value>,
    {
        Expr::col((self.entity_name(), *self)).if_null(v)
    }

    bind_vec_func!(is_in);
    bind_vec_func!(is_not_in);

    /// Postgres only.
    /// ```
    /// use sea_orm::{DbBackend, entity::*, query::*, tests_cfg::cake};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .filter(cake::Column::Id.eq_any(vec![4, 5]))
    ///         .build(DbBackend::Postgres)
    ///         .to_string(),
    ///     r#"SELECT "cake"."id", "cake"."name" FROM "cake" WHERE "cake"."id" = ANY(ARRAY [4,5])"#
    /// );
    /// ```
    #[cfg(feature = "postgres-array")]
    fn eq_any<V, I>(&self, v: I) -> SimpleExpr
    where
        V: Into<Value> + sea_query::ValueType + sea_query::with_array::NotU8,
        I: IntoIterator<Item = V>,
    {
        use sea_query::extension::postgres::PgFunc;

        let vec: Vec<_> = v.into_iter().collect();
        Expr::col((self.entity_name(), *self)).eq(PgFunc::any(vec))
    }

    bind_subquery_func!(in_subquery);
    bind_subquery_func!(not_in_subquery);

    /// Construct a [`SimpleExpr::Column`] wrapped in [`Expr`].
    fn into_expr(self) -> Expr {
        self.into_simple_expr()
    }

    /// Construct a returning [`Expr`].
    #[allow(clippy::match_single_binding)]
    fn into_returning_expr(self, db_backend: DbBackend) -> Expr {
        match db_backend {
            _ => Expr::col(self),
        }
    }

    /// Cast column expression used in select statement.
    /// It only cast database enum as text if it's an enum column.
    fn select_as(&self, expr: Expr) -> SimpleExpr {
        self.select_enum_as(expr)
    }

    /// Cast enum column as text; do nothing if `self` is not an enum.
    fn select_enum_as(&self, expr: Expr) -> SimpleExpr {
        cast_enum_as(expr, self, |col, _, col_type| {
            let type_name = match col_type {
                ColumnType::Array(_) => TextArray.into_iden(),
                _ => Text.into_iden(),
            };
            col.as_enum(type_name)
        })
    }

    /// Cast value of a column into the correct type for database storage.
    /// It only cast text as enum type if it's an enum column.
    fn save_as(&self, val: Expr) -> SimpleExpr {
        self.save_enum_as(val)
    }

    /// Cast value of an enum column as enum type; do nothing if `self` is not an enum.
    /// Will also transform `Array(Vec<Json>)` into `Json(Vec<Json>)` if the column type is `Json`.
    fn save_enum_as(&self, val: Expr) -> SimpleExpr {
        cast_enum_as(val, self, |col, enum_name, col_type| {
            let type_name = match col_type {
                ColumnType::Array(_) => Alias::new(format!("{enum_name}[]")).into_iden(),
                _ => enum_name,
            };
            col.as_enum(type_name)
        })
    }
}

/// SeaORM's utility methods that act on [ColumnType]
pub trait ColumnTypeTrait {
    /// Instantiate a new [ColumnDef]
    fn def(self) -> ColumnDef;

    /// Get the name of the enum if this is a enum column
    fn get_enum_name(&self) -> Option<&DynIden>;
}

impl ColumnTypeTrait for ColumnType {
    fn def(self) -> ColumnDef {
        ColumnDef {
            col_type: self,
            null: false,
            unique: false,
            indexed: false,
            default: None,
            comment: None,
            unique_key: None,
        }
    }

    fn get_enum_name(&self) -> Option<&DynIden> {
        enum_name(self)
    }
}

impl ColumnTypeTrait for ColumnDef {
    fn def(self) -> ColumnDef {
        self
    }

    fn get_enum_name(&self) -> Option<&DynIden> {
        enum_name(&self.col_type)
    }
}

fn enum_name(col_type: &ColumnType) -> Option<&DynIden> {
    match col_type {
        ColumnType::Enum { name, .. } => Some(name),
        ColumnType::Array(col_type) => enum_name(col_type),
        _ => None,
    }
}

struct Text;
struct TextArray;

impl Iden for Text {
    fn quoted(&self) -> Cow<'static, str> {
        Cow::Borrowed("text")
    }

    fn unquoted(&self) -> &str {
        match self.quoted() {
            Cow::Borrowed(s) => s,
            _ => unreachable!(),
        }
    }
}

impl Iden for TextArray {
    fn quoted(&self) -> Cow<'static, str> {
        // This is Postgres only and it has a special handling for quoting this
        Cow::Borrowed("text[]")
    }

    fn unquoted(&self) -> &str {
        match self.quoted() {
            Cow::Borrowed(s) => s,
            _ => unreachable!(),
        }
    }
}

fn cast_enum_as<C, F>(expr: Expr, col: &C, f: F) -> SimpleExpr
where
    C: ColumnTrait,
    F: Fn(Expr, DynIden, &ColumnType) -> SimpleExpr,
{
    let col_def = col.def();
    let col_type = col_def.get_column_type();

    match col_type {
        #[cfg(all(feature = "with-json", feature = "postgres-array"))]
        ColumnType::Json | ColumnType::JsonBinary => {
            use sea_query::ArrayType;
            use serde_json::Value as Json;

            match expr {
                SimpleExpr::Value(Value::Array(ArrayType::Json, Some(json_vec))) => {
                    // flatten Array(Vec<Json>) into Json
                    let json_vec: Vec<Json> = json_vec
                        .into_iter()
                        .filter_map(|val| match val {
                            Value::Json(Some(json)) => Some(json),
                            _ => None,
                        })
                        .collect();
                    SimpleExpr::Value(Value::Json(Some(json_vec.into())))
                }
                SimpleExpr::Value(Value::Array(ArrayType::Json, None)) => {
                    SimpleExpr::Value(Value::Json(None))
                }
                _ => expr,
            }
        }
        _ => match col_type.get_enum_name() {
            Some(enum_name) => f(expr, SeaRc::clone(enum_name), col_type),
            None => expr,
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ColumnTrait, Condition, DbBackend, EntityTrait, QueryFilter, QueryTrait, tests_cfg::*,
    };
    use sea_query::Query;

    #[test]
    fn test_in_subquery_1() {
        assert_eq!(
            cake::Entity::find()
                .filter(
                    Condition::any().add(
                        cake::Column::Id.in_subquery(
                            Query::select()
                                .expr(cake::Column::Id.max())
                                .from(cake::Entity)
                                .to_owned()
                        )
                    )
                )
                .build(DbBackend::MySql)
                .to_string(),
            [
                "SELECT `cake`.`id`, `cake`.`name` FROM `cake`",
                "WHERE `cake`.`id` IN (SELECT MAX(`cake`.`id`) FROM `cake`)",
            ]
            .join(" ")
        );
    }

    #[test]
    fn test_in_subquery_2() {
        assert_eq!(
            cake::Entity::find()
                .filter(
                    Condition::any().add(
                        cake::Column::Id.in_subquery(
                            Query::select()
                                .column(cake_filling::Column::CakeId)
                                .from(cake_filling::Entity)
                                .to_owned()
                        )
                    )
                )
                .build(DbBackend::MySql)
                .to_string(),
            [
                "SELECT `cake`.`id`, `cake`.`name` FROM `cake`",
                "WHERE `cake`.`id` IN (SELECT `cake_id` FROM `cake_filling`)",
            ]
            .join(" ")
        );
    }

    #[test]
    #[cfg(feature = "macros")]
    fn select_as_1() {
        use crate::{ActiveModelTrait, ActiveValue, Update};

        mod hello_expanded {
            use crate as sea_orm;
            use crate::entity::prelude::*;
            use crate::sea_query::{Expr, ExprTrait, SimpleExpr};

            #[derive(Copy, Clone, Default, Debug, DeriveEntity)]
            pub struct Entity;

            impl EntityName for Entity {
                fn table_name(&self) -> &'static str {
                    "hello"
                }
            }

            #[derive(Clone, Debug, PartialEq, Eq, DeriveModel, DeriveActiveModel)]
            pub struct Model {
                pub id: i32,
                #[sea_orm(enum_name = "One1")]
                pub one: i32,
                pub two: i32,
                #[sea_orm(enum_name = "Three3")]
                pub three: i32,
            }

            #[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
            pub enum Column {
                Id,
                One1,
                Two,
                Three3,
            }

            impl ColumnTrait for Column {
                type EntityName = Entity;

                fn def(&self) -> ColumnDef {
                    match self {
                        Column::Id => ColumnType::Integer.def(),
                        Column::One1 => ColumnType::Integer.def(),
                        Column::Two => ColumnType::Integer.def(),
                        Column::Three3 => ColumnType::Integer.def(),
                    }
                }

                fn select_as(&self, expr: Expr) -> SimpleExpr {
                    match self {
                        Self::Two => expr.cast_as("integer"),
                        _ => self.select_enum_as(expr),
                    }
                }
            }

            #[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
            pub enum PrimaryKey {
                Id,
            }

            impl PrimaryKeyTrait for PrimaryKey {
                type ValueType = i32;

                fn auto_increment() -> bool {
                    true
                }
            }

            #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
            pub enum Relation {}

            impl ActiveModelBehavior for ActiveModel {}
        }

        #[allow(clippy::enum_variant_names)]
        mod hello_compact {
            use crate as sea_orm;
            use crate::entity::prelude::*;

            #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
            #[sea_orm(table_name = "hello")]
            pub struct Model {
                #[sea_orm(primary_key)]
                pub id: i32,
                #[sea_orm(enum_name = "One1")]
                pub one: i32,
                #[sea_orm(select_as = "integer")]
                pub two: i32,
                #[sea_orm(enum_name = "Three3")]
                pub three: i32,
            }

            #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
            pub enum Relation {}

            impl ActiveModelBehavior for ActiveModel {}
        }

        fn assert_it<E, A>(active_model: A)
        where
            E: EntityTrait,
            A: ActiveModelTrait<Entity = E>,
        {
            assert_eq!(
                E::find().build(DbBackend::Postgres).to_string(),
                r#"SELECT "hello"."id", "hello"."one1", CAST("hello"."two" AS integer), "hello"."three3" FROM "hello""#,
            );
            assert_eq!(
                Update::one(active_model)
                    .build(DbBackend::Postgres)
                    .to_string(),
                r#"UPDATE "hello" SET "one1" = 1, "two" = 2, "three3" = 3 WHERE "hello"."id" = 1"#,
            );
        }

        assert_it(hello_expanded::ActiveModel {
            id: ActiveValue::set(1),
            one: ActiveValue::set(1),
            two: ActiveValue::set(2),
            three: ActiveValue::set(3),
        });
        assert_it(hello_compact::ActiveModel {
            id: ActiveValue::set(1),
            one: ActiveValue::set(1),
            two: ActiveValue::set(2),
            three: ActiveValue::set(3),
        });
    }

    #[test]
    #[cfg(feature = "macros")]
    fn save_as_1() {
        use crate::{ActiveModelTrait, ActiveValue, Update};

        mod hello_expanded {
            use crate as sea_orm;
            use crate::entity::prelude::*;
            use crate::sea_query::{Expr, ExprTrait, SimpleExpr};

            #[derive(Copy, Clone, Default, Debug, DeriveEntity)]
            pub struct Entity;

            impl EntityName for Entity {
                fn table_name(&self) -> &'static str {
                    "hello"
                }
            }

            #[derive(Clone, Debug, PartialEq, Eq, DeriveModel, DeriveActiveModel)]
            pub struct Model {
                pub id: i32,
                #[sea_orm(enum_name = "One1")]
                pub one: i32,
                pub two: i32,
                #[sea_orm(enum_name = "Three3")]
                pub three: i32,
            }

            #[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
            pub enum Column {
                Id,
                One1,
                Two,
                Three3,
            }

            impl ColumnTrait for Column {
                type EntityName = Entity;

                fn def(&self) -> ColumnDef {
                    match self {
                        Column::Id => ColumnType::Integer.def(),
                        Column::One1 => ColumnType::Integer.def(),
                        Column::Two => ColumnType::Integer.def(),
                        Column::Three3 => ColumnType::Integer.def(),
                    }
                }

                fn save_as(&self, val: Expr) -> SimpleExpr {
                    match self {
                        Self::Two => val.cast_as("text"),
                        _ => self.save_enum_as(val),
                    }
                }
            }

            #[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
            pub enum PrimaryKey {
                Id,
            }

            impl PrimaryKeyTrait for PrimaryKey {
                type ValueType = i32;

                fn auto_increment() -> bool {
                    true
                }
            }

            #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
            pub enum Relation {}

            impl ActiveModelBehavior for ActiveModel {}
        }

        #[allow(clippy::enum_variant_names)]
        mod hello_compact {
            use crate as sea_orm;
            use crate::entity::prelude::*;

            #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
            #[sea_orm(table_name = "hello")]
            pub struct Model {
                #[sea_orm(primary_key)]
                pub id: i32,
                #[sea_orm(enum_name = "One1")]
                pub one: i32,
                #[sea_orm(save_as = "text")]
                pub two: i32,
                #[sea_orm(enum_name = "Three3")]
                pub three: i32,
            }

            #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
            pub enum Relation {}

            impl ActiveModelBehavior for ActiveModel {}
        }

        fn assert_it<E, A>(active_model: A)
        where
            E: EntityTrait,
            A: ActiveModelTrait<Entity = E>,
        {
            assert_eq!(
                E::find().build(DbBackend::Postgres).to_string(),
                r#"SELECT "hello"."id", "hello"."one1", "hello"."two", "hello"."three3" FROM "hello""#,
            );
            assert_eq!(
                Update::one(active_model)
                    .build(DbBackend::Postgres)
                    .to_string(),
                r#"UPDATE "hello" SET "one1" = 1, "two" = CAST(2 AS text), "three3" = 3 WHERE "hello"."id" = 1"#,
            );
        }

        assert_it(hello_expanded::ActiveModel {
            id: ActiveValue::set(1),
            one: ActiveValue::set(1),
            two: ActiveValue::set(2),
            three: ActiveValue::set(3),
        });
        assert_it(hello_compact::ActiveModel {
            id: ActiveValue::set(1),
            one: ActiveValue::set(1),
            two: ActiveValue::set(2),
            three: ActiveValue::set(3),
        });
    }

    #[test]
    #[cfg(feature = "macros")]
    fn select_as_and_value_1() {
        use crate::{ActiveModelTrait, ActiveValue, Update};

        mod hello_expanded {
            use crate as sea_orm;
            use crate::entity::prelude::*;
            use crate::sea_query::{Expr, ExprTrait, SimpleExpr};

            #[derive(Copy, Clone, Default, Debug, DeriveEntity)]
            pub struct Entity;

            impl EntityName for Entity {
                fn table_name(&self) -> &'static str {
                    "hello"
                }
            }

            #[derive(Clone, Debug, PartialEq, Eq, DeriveModel, DeriveActiveModel)]
            pub struct Model {
                pub id: i32,
                #[sea_orm(enum_name = "One1")]
                pub one: i32,
                pub two: i32,
                #[sea_orm(enum_name = "Three3")]
                pub three: i32,
            }

            #[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
            pub enum Column {
                Id,
                One1,
                Two,
                Three3,
            }

            impl ColumnTrait for Column {
                type EntityName = Entity;

                fn def(&self) -> ColumnDef {
                    match self {
                        Column::Id => ColumnType::Integer.def(),
                        Column::One1 => ColumnType::Integer.def(),
                        Column::Two => ColumnType::Integer.def(),
                        Column::Three3 => ColumnType::Integer.def(),
                    }
                }

                fn select_as(&self, expr: Expr) -> SimpleExpr {
                    match self {
                        Self::Two => expr.cast_as("integer"),
                        _ => self.select_enum_as(expr),
                    }
                }

                fn save_as(&self, val: Expr) -> SimpleExpr {
                    match self {
                        Self::Two => val.cast_as("text"),
                        _ => self.save_enum_as(val),
                    }
                }
            }

            #[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
            pub enum PrimaryKey {
                Id,
            }

            impl PrimaryKeyTrait for PrimaryKey {
                type ValueType = i32;

                fn auto_increment() -> bool {
                    true
                }
            }

            #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
            pub enum Relation {}

            impl ActiveModelBehavior for ActiveModel {}
        }

        #[allow(clippy::enum_variant_names)]
        mod hello_compact {
            use crate as sea_orm;
            use crate::entity::prelude::*;

            #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
            #[sea_orm(table_name = "hello")]
            pub struct Model {
                #[sea_orm(primary_key)]
                pub id: i32,
                #[sea_orm(enum_name = "One1")]
                pub one: i32,
                #[sea_orm(select_as = "integer", save_as = "text")]
                pub two: i32,
                #[sea_orm(enum_name = "Three3")]
                pub three: i32,
            }

            #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
            pub enum Relation {}

            impl ActiveModelBehavior for ActiveModel {}
        }

        fn assert_it<E, A>(active_model: A)
        where
            E: EntityTrait,
            A: ActiveModelTrait<Entity = E>,
        {
            assert_eq!(
                E::find().build(DbBackend::Postgres).to_string(),
                r#"SELECT "hello"."id", "hello"."one1", CAST("hello"."two" AS integer), "hello"."three3" FROM "hello""#,
            );
            assert_eq!(
                Update::one(active_model)
                    .build(DbBackend::Postgres)
                    .to_string(),
                r#"UPDATE "hello" SET "one1" = 1, "two" = CAST(2 AS text), "three3" = 3 WHERE "hello"."id" = 1"#,
            );
        }

        assert_it(hello_expanded::ActiveModel {
            id: ActiveValue::set(1),
            one: ActiveValue::set(1),
            two: ActiveValue::set(2),
            three: ActiveValue::set(3),
        });
        assert_it(hello_compact::ActiveModel {
            id: ActiveValue::set(1),
            one: ActiveValue::set(1),
            two: ActiveValue::set(2),
            three: ActiveValue::set(3),
        });
    }
}
