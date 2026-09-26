use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "geo")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,
    pub name: String,
    #[sea_orm(
        column_type = r#"custom("point")"#,
        select_as = "text",
        save_as = "point"
    )]
    pub location: Geo<PgPoint>,
    #[sea_orm(
        column_type = r#"custom("polygon")"#,
        select_as = "text",
        save_as = "polygon"
    )]
    pub boundary: Geo<PgPolygon>,
    #[sea_orm(
        column_type = r#"custom("box")"#,
        select_as = "text",
        save_as = "box"
    )]
    pub bounds: Geo<PgBox>,
    #[sea_orm(
        column_type = r#"custom("circle")"#,
        select_as = "text",
        save_as = "circle"
    )]
    pub area: Geo<PgCircle>,
    #[sea_orm(
        column_type = r#"custom("lseg")"#,
        select_as = "text",
        save_as = "lseg"
    )]
    pub segment: Geo<PgLSeg>,
    #[sea_orm(
        column_type = r#"custom("line")"#,
        select_as = "text",
        save_as = "line"
    )]
    pub line: Geo<PgLine>,
    #[sea_orm(
        column_type = r#"custom("path")"#,
        select_as = "text",
        save_as = "path"
    )]
    pub route: Geo<PgPath>,
    #[sea_orm(
        column_type = r#"custom("point")"#,
        select_as = "text",
        save_as = "point",
        nullable
    )]
    pub optional_point: Option<Geo<PgPoint>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
