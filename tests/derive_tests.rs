mod from_query_result {
    use sea_orm::{FromQueryResult, TryGetable};

    #[derive(FromQueryResult)]
    struct SimpleTest {
        _foo: i32,
        _bar: String,
    }

    #[derive(FromQueryResult)]
    struct GenericTest<T> {
        _foo: i32,
        _bar: T,
    }

    #[derive(FromQueryResult)]
    struct DoubleGenericTest<T, F> {
        _foo: T,
        _bar: F,
    }

    #[derive(FromQueryResult)]
    struct BoundsGenericTest<T: Copy + Clone + 'static> {
        _foo: T,
    }

    #[derive(FromQueryResult)]
    struct WhereGenericTest<T>
    where
        T: Copy + Clone + 'static,
    {
        _foo: T,
    }

    #[derive(FromQueryResult)]
    struct AlreadySpecifiedBoundsGenericTest<T: TryGetable> {
        _foo: T,
    }

    #[derive(FromQueryResult)]
    struct MixedGenericTest<T: Clone, F>
    where
        F: Copy + Clone + 'static,
    {
        _foo: T,
        _bar: F,
    }
}

mod partial_model {
    use entity::{Column, Entity};
    use sea_orm::{ColumnTrait, DerivePartialModel, FromQueryResult};
    use sea_query::Expr;

    mod entity {
        use sea_orm::prelude::*;

        #[derive(Debug, Clone, DeriveEntityModel)]
        #[sea_orm(table_name = "foo_table")]
        pub struct Model {
            #[sea_orm(primary_key)]
            id: i32,
            foo: i32,
            bar: String,
            foo2: bool,
            bar2: f64,
        }

        #[derive(Debug, DeriveRelation, EnumIter)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}
    }

    #[derive(FromQueryResult, DerivePartialModel)]
    #[sea_orm(entity = "Entity")]
    struct SimpleTest {
        _foo: i32,
        _bar: String,
    }

    #[derive(FromQueryResult, DerivePartialModel)]
    #[sea_orm(entity = "Entity")]
    struct FieldFromDiffNameColumnTest {
        #[sea_orm(from_col = "foo2")]
        _foo: i32,
        #[sea_orm(from_col = "bar2")]
        _bar: String,
    }

    #[derive(FromQueryResult, DerivePartialModel)]
    struct FieldFromExpr {
        #[sea_orm(from_expr = "Column::Bar2.sum()")]
        _foo: f64,
        #[sea_orm(from_expr = "Expr::col(Column::Id).equals(Column::Foo)")]
        _bar: bool,
    }
}
