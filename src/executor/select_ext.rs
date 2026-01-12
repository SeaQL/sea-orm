use crate::{
    ConnectionTrait, DbErr, EntityTrait, Select, SelectFive, SelectFour, SelectSix, SelectThree,
    SelectTwo, Selector, SelectorRaw, SelectorTrait, Topology,
};
use sea_query::{Expr, SelectStatement};

// TODO: Move count here
#[async_trait::async_trait]
/// Helper trait for selectors with convenient methods
pub trait SelectExt {
    /// This method is unstable and is only used for internal testing.
    /// It may be removed in the future.
    #[doc(hidden)]
    fn exists_query(self) -> SelectStatement;
    /// Check if any records exist
    async fn exists<C>(self, db: &C) -> Result<bool, DbErr>
    where
        C: ConnectionTrait,
        Self: Send + Sized,
    {
        let stmt = self.exists_query();
        Ok(db.query_one(&stmt).await?.is_some())
    }
}

fn into_exists_query(mut stmt: SelectStatement) -> SelectStatement {
    stmt.clear_selects();
    // Expr::Custom has fewer branches, but this may not have any significant impact on performance.
    stmt.expr(Expr::cust("1"));
    stmt.reset_limit();
    stmt.reset_offset();
    stmt.clear_order_by();
    stmt
}

impl<S> SelectExt for Selector<S>
where
    S: SelectorTrait,
{
    fn exists_query(self) -> SelectStatement {
        into_exists_query(self.query)
    }
}

#[async_trait::async_trait]
impl<S> SelectExt for SelectorRaw<S>
where
    S: SelectorTrait,
{
    fn exists_query(self) -> SelectStatement {
        let stmt = self.stmt;
        let sub_query_sql = stmt.sql.trim().trim_end_matches(';').trim();
        let exists_sql = format!("1 FROM ({sub_query_sql}) AS sub_query LIMIT 1");

        let mut query = SelectStatement::new();
        query.expr(if let Some(values) = stmt.values {
            Expr::cust_with_values(exists_sql, values.0)
        } else {
            Expr::cust(exists_sql)
        });
        query
    }
}

impl<E> SelectExt for Select<E>
where
    E: EntityTrait,
{
    fn exists_query(self) -> SelectStatement {
        into_exists_query(self.query)
    }
}

impl<E, F> SelectExt for SelectTwo<E, F>
where
    E: EntityTrait,
    F: EntityTrait,
{
    fn exists_query(self) -> SelectStatement {
        into_exists_query(self.query)
    }
}

impl<E, F, G, TOP> SelectExt for SelectThree<E, F, G, TOP>
where
    E: EntityTrait,
    F: EntityTrait,
    G: EntityTrait,
    TOP: Topology,
{
    fn exists_query(self) -> SelectStatement {
        into_exists_query(self.query)
    }
}

impl<E, F, G, H, TOP> SelectExt for SelectFour<E, F, G, H, TOP>
where
    E: EntityTrait,
    F: EntityTrait,
    G: EntityTrait,
    H: EntityTrait,
    TOP: Topology,
{
    fn exists_query(self) -> SelectStatement {
        into_exists_query(self.query)
    }
}

impl<E, F, G, H, I, TOP> SelectExt for SelectFive<E, F, G, H, I, TOP>
where
    E: EntityTrait,
    F: EntityTrait,
    G: EntityTrait,
    H: EntityTrait,
    I: EntityTrait,
    TOP: Topology,
{
    fn exists_query(self) -> SelectStatement {
        into_exists_query(self.query)
    }
}

impl<E, F, G, H, I, J, TOP> SelectExt for SelectSix<E, F, G, H, I, J, TOP>
where
    E: EntityTrait,
    F: EntityTrait,
    G: EntityTrait,
    H: EntityTrait,
    I: EntityTrait,
    J: EntityTrait,
    TOP: Topology,
{
    fn exists_query(self) -> SelectStatement {
        into_exists_query(self.query)
    }
}

#[cfg(test)]
mod tests {
    use super::SelectExt;
    use crate::entity::prelude::*;
    use crate::{DbBackend, QueryOrder, QuerySelect, Statement, tests_cfg::*};

    #[test]
    fn exists_query_select_basic() {
        let stmt = fruit::Entity::find().exists_query();
        let sql = DbBackend::Postgres.build(&stmt).to_string();
        assert_eq!(sql, r#"SELECT 1 FROM "fruit""#);
    }

    #[test]
    fn exists_query_select_strips_limit_offset_order() {
        let stmt = fruit::Entity::find()
            .filter(fruit::Column::Id.gt(1))
            .order_by_asc(fruit::Column::Id)
            .limit(2)
            .offset(4)
            .exists_query();

        let sql = DbBackend::Postgres.build(&stmt).to_string();
        assert_eq!(sql, r#"SELECT 1 FROM "fruit" WHERE "fruit"."id" > 1"#);
    }

    #[test]
    fn exists_query_selector_basic() {
        let stmt = fruit::Entity::find()
            .into_model::<fruit::Model>()
            .exists_query();

        let sql = DbBackend::Postgres.build(&stmt).to_string();
        assert_eq!(sql, r#"SELECT 1 FROM "fruit""#);
    }

    #[test]
    fn exists_query_selector_complex() {
        let stmt = fruit::Entity::find()
            .filter(fruit::Column::Id.gt(1))
            .order_by_desc(fruit::Column::Id)
            .limit(2)
            .offset(4)
            .into_model::<fruit::Model>()
            .exists_query();

        let sql = DbBackend::Postgres.build(&stmt).to_string();
        assert_eq!(sql, r#"SELECT 1 FROM "fruit" WHERE "fruit"."id" > 1"#);
    }

    #[test]
    fn exists_query_selector_raw_simple() {
        let raw_stmt =
            Statement::from_string(DbBackend::Postgres, r#"SELECT "fruit"."id" FROM "fruit""#);
        let stmt = fruit::Entity::find().from_raw_sql(raw_stmt).exists_query();

        let sql = DbBackend::Postgres.build(&stmt).to_string();
        assert_eq!(
            sql,
            r#"SELECT 1 FROM (SELECT "fruit"."id" FROM "fruit") AS sub_query LIMIT 1"#
        );
    }

    #[test]
    fn exists_query_selector_raw_complex() {
        let raw_stmt = Statement::from_string(
            DbBackend::Postgres,
            r#"SELECT "fruit"."id" FROM "fruit" WHERE "fruit"."id" > 1 ORDER BY "fruit"."id" DESC LIMIT 5 OFFSET 2"#,
        );
        let stmt = fruit::Entity::find().from_raw_sql(raw_stmt).exists_query();

        let sql = DbBackend::Postgres.build(&stmt).to_string();
        assert_eq!(
            sql,
            r#"SELECT 1 FROM (SELECT "fruit"."id" FROM "fruit" WHERE "fruit"."id" > 1 ORDER BY "fruit"."id" DESC LIMIT 5 OFFSET 2) AS sub_query LIMIT 1"#
        );
    }

    #[test]
    fn exists_query_select_two_simple() {
        let stmt = cake::Entity::find()
            .find_also_related(fruit::Entity)
            .exists_query();

        let sql = DbBackend::Postgres.build(&stmt).to_string();
        assert_eq!(
            sql,
            r#"SELECT 1 FROM "cake" LEFT JOIN "fruit" ON "cake"."id" = "fruit"."cake_id""#
        );
    }

    #[test]
    fn exists_query_select_two_complex() {
        let stmt = cake::Entity::find()
            .find_also_related(fruit::Entity)
            .filter(cake::Column::Id.gt(1))
            .order_by_desc(cake::Column::Id)
            .limit(2)
            .offset(4)
            .exists_query();

        let sql = DbBackend::Postgres.build(&stmt).to_string();
        assert_eq!(
            sql,
            [
                r#"SELECT 1 FROM "cake""#,
                r#"LEFT JOIN "fruit" ON "cake"."id" = "fruit"."cake_id""#,
                r#"WHERE "cake"."id" > 1"#,
            ]
            .join(" ")
        );
    }

    #[test]
    fn exists_query_select_three_simple() {
        let stmt = cake_filling::Entity::find()
            .find_also_related(cake::Entity)
            .find_also(cake_filling::Entity, filling::Entity)
            .exists_query();

        let sql = DbBackend::Postgres.build(&stmt).to_string();
        assert_eq!(
            sql,
            [
                r#"SELECT 1 FROM "cake_filling""#,
                r#"LEFT JOIN "cake" ON "cake_filling"."cake_id" = "cake"."id""#,
                r#"LEFT JOIN "filling" ON "cake_filling"."filling_id" = "filling"."id""#,
            ]
            .join(" ")
        );
    }

    #[test]
    fn exists_query_select_three_complex() {
        let stmt = cake_filling::Entity::find()
            .find_also_related(cake::Entity)
            .find_also(cake_filling::Entity, filling::Entity)
            .filter(cake_filling::Column::CakeId.gt(1))
            .order_by_desc(cake_filling::Column::CakeId)
            .limit(2)
            .offset(4)
            .exists_query();

        let sql = DbBackend::Postgres.build(&stmt).to_string();
        assert_eq!(
            sql,
            [
                r#"SELECT 1 FROM "cake_filling""#,
                r#"LEFT JOIN "cake" ON "cake_filling"."cake_id" = "cake"."id""#,
                r#"LEFT JOIN "filling" ON "cake_filling"."filling_id" = "filling"."id""#,
                r#"WHERE "cake_filling"."cake_id" > 1"#,
            ]
            .join(" ")
        );
    }

    #[test]
    fn exists_query_select_four_simple() {
        let stmt = cake_filling::Entity::find()
            .find_also_related(cake::Entity)
            .find_also(cake_filling::Entity, filling::Entity)
            .find_also(filling::Entity, ingredient::Entity)
            .exists_query();

        let sql = DbBackend::Postgres.build(&stmt).to_string();
        assert_eq!(
            sql,
            [
                r#"SELECT 1 FROM "cake_filling""#,
                r#"LEFT JOIN "cake" ON "cake_filling"."cake_id" = "cake"."id""#,
                r#"LEFT JOIN "filling" ON "cake_filling"."filling_id" = "filling"."id""#,
                r#"LEFT JOIN "ingredient" ON "filling"."id" = "ingredient"."filling_id""#,
            ]
            .join(" ")
        );
    }

    #[test]
    fn exists_query_select_four_complex() {
        let stmt = cake_filling::Entity::find()
            .find_also_related(cake::Entity)
            .find_also(cake_filling::Entity, filling::Entity)
            .find_also(filling::Entity, ingredient::Entity)
            .filter(cake_filling::Column::CakeId.gt(1))
            .order_by_desc(cake_filling::Column::CakeId)
            .limit(2)
            .offset(4)
            .exists_query();

        let sql = DbBackend::Postgres.build(&stmt).to_string();
        assert_eq!(
            sql,
            [
                r#"SELECT 1 FROM "cake_filling""#,
                r#"LEFT JOIN "cake" ON "cake_filling"."cake_id" = "cake"."id""#,
                r#"LEFT JOIN "filling" ON "cake_filling"."filling_id" = "filling"."id""#,
                r#"LEFT JOIN "ingredient" ON "filling"."id" = "ingredient"."filling_id""#,
                r#"WHERE "cake_filling"."cake_id" > 1"#,
            ]
            .join(" ")
        );
    }

    #[test]
    fn exists_query_select_five_simple() {
        let stmt = cake_filling::Entity::find()
            .find_also_related(cake::Entity)
            .find_also(cake_filling::Entity, filling::Entity)
            .find_also(filling::Entity, ingredient::Entity)
            .find_also(cake_filling::Entity, cake_filling_price::Entity)
            .exists_query();

        let sql = DbBackend::Postgres.build(&stmt).to_string();
        assert_eq!(
            sql,
            [
                r#"SELECT 1 FROM "cake_filling""#,
                r#"LEFT JOIN "cake" ON "cake_filling"."cake_id" = "cake"."id""#,
                r#"LEFT JOIN "filling" ON "cake_filling"."filling_id" = "filling"."id""#,
                r#"LEFT JOIN "ingredient" ON "filling"."id" = "ingredient"."filling_id""#,
                r#"LEFT JOIN "public"."cake_filling_price" ON "cake_filling"."cake_id" = "cake_filling_price"."cake_id" AND "cake_filling"."filling_id" = "cake_filling_price"."filling_id""#,
            ]
            .join(" ")
        );
    }

    #[test]
    fn exists_query_select_five_complex() {
        let stmt = cake_filling::Entity::find()
            .find_also_related(cake::Entity)
            .find_also(cake_filling::Entity, filling::Entity)
            .find_also(filling::Entity, ingredient::Entity)
            .find_also(cake_filling::Entity, cake_filling_price::Entity)
            .filter(cake_filling::Column::CakeId.gt(1))
            .order_by_desc(cake_filling::Column::CakeId)
            .limit(2)
            .offset(4)
            .exists_query();

        let sql = DbBackend::Postgres.build(&stmt).to_string();
        assert_eq!(
            sql,
            [
                r#"SELECT 1 FROM "cake_filling""#,
                r#"LEFT JOIN "cake" ON "cake_filling"."cake_id" = "cake"."id""#,
                r#"LEFT JOIN "filling" ON "cake_filling"."filling_id" = "filling"."id""#,
                r#"LEFT JOIN "ingredient" ON "filling"."id" = "ingredient"."filling_id""#,
                r#"LEFT JOIN "public"."cake_filling_price" ON "cake_filling"."cake_id" = "cake_filling_price"."cake_id" AND "cake_filling"."filling_id" = "cake_filling_price"."filling_id""#,
                r#"WHERE "cake_filling"."cake_id" > 1"#,
            ]
            .join(" ")
        );
    }

    #[test]
    fn exists_query_select_six_simple() {
        let stmt = cake_filling::Entity::find()
            .find_also_related(cake::Entity)
            .find_also(cake_filling::Entity, filling::Entity)
            .find_also(filling::Entity, ingredient::Entity)
            .find_also(cake_filling::Entity, cake_filling_price::Entity)
            .find_also(filling::Entity, cake_compact::Entity)
            .exists_query();

        let sql = DbBackend::Postgres.build(&stmt).to_string();
        assert_eq!(
            sql,
            [
                r#"SELECT 1 FROM "cake_filling""#,
                r#"LEFT JOIN "cake" ON "cake_filling"."cake_id" = "cake"."id""#,
                r#"LEFT JOIN "filling" ON "cake_filling"."filling_id" = "filling"."id""#,
                r#"LEFT JOIN "ingredient" ON "filling"."id" = "ingredient"."filling_id""#,
                r#"LEFT JOIN "public"."cake_filling_price" ON "cake_filling"."cake_id" = "cake_filling_price"."cake_id" AND "cake_filling"."filling_id" = "cake_filling_price"."filling_id""#,
                r#"LEFT JOIN "cake_filling" ON "filling"."id" = "cake_filling"."filling_id""#,
                r#"LEFT JOIN "cake" ON "cake_filling"."cake_id" = "cake"."id""#,
            ]
            .join(" ")
        );
    }

    #[test]
    fn exists_query_select_six_complex() {
        let stmt = cake_filling::Entity::find()
            .find_also_related(cake::Entity)
            .find_also(cake_filling::Entity, filling::Entity)
            .find_also(filling::Entity, ingredient::Entity)
            .find_also(cake_filling::Entity, cake_filling_price::Entity)
            .find_also(filling::Entity, cake_compact::Entity)
            .filter(cake_filling::Column::CakeId.gt(1))
            .order_by_desc(cake_filling::Column::CakeId)
            .limit(2)
            .offset(4)
            .exists_query();

        let sql = DbBackend::Postgres.build(&stmt).to_string();
        assert_eq!(
            sql,
            [
                r#"SELECT 1 FROM "cake_filling""#,
                r#"LEFT JOIN "cake" ON "cake_filling"."cake_id" = "cake"."id""#,
                r#"LEFT JOIN "filling" ON "cake_filling"."filling_id" = "filling"."id""#,
                r#"LEFT JOIN "ingredient" ON "filling"."id" = "ingredient"."filling_id""#,
                r#"LEFT JOIN "public"."cake_filling_price" ON "cake_filling"."cake_id" = "cake_filling_price"."cake_id" AND "cake_filling"."filling_id" = "cake_filling_price"."filling_id""#,
                r#"LEFT JOIN "cake_filling" ON "filling"."id" = "cake_filling"."filling_id""#,
                r#"LEFT JOIN "cake" ON "cake_filling"."cake_id" = "cake"."id""#,
                r#"WHERE "cake_filling"."cake_id" > 1"#,
            ]
            .join(" ")
        );
    }
}
