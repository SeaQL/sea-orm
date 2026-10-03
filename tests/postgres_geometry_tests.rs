#![allow(unused_imports, dead_code)]

pub mod common;

use common::features::*;
use pretty_assertions::assert_eq;
use sea_orm::{
    ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement, entity::prelude::*, entity::*,
};

#[sea_orm_macros::test]
#[cfg(all(feature = "sqlx-postgres", feature = "postgres-geometry"))]
async fn main() -> Result<(), DbErr> {
    let ctx = common::TestContext::new("postgres_geometry_tests").await;

    // Built-in geometric types — no PostGIS extension required.
    ctx.db
        .execute_unprepared(
            r#"CREATE TABLE "geo" (
                "id" integer PRIMARY KEY NOT NULL,
                "name" varchar NOT NULL,
                "location" point NOT NULL,
                "boundary" polygon NOT NULL,
                "bounds" box NOT NULL,
                "area" circle NOT NULL,
                "segment" lseg NOT NULL,
                "line" line NOT NULL,
                "route" path NOT NULL,
                "optional_point" point
            )"#,
        )
        .await?;

    round_trip(&ctx.db).await?;
    spatial_query(&ctx.db).await?;

    ctx.delete().await;
    Ok(())
}

#[cfg(all(feature = "sqlx-postgres", feature = "postgres-geometry"))]
async fn round_trip(db: &DatabaseConnection) -> Result<(), DbErr> {
    let model = geo::Model {
        id: 1,
        name: "sleipner".to_owned(),
        location: Geo::new(PgPoint { x: 1.5, y: 2.5 }),
        boundary: Geo::new(PgPolygon {
            points: vec![
                PgPoint { x: 0.0, y: 0.0 },
                PgPoint { x: 0.0, y: 1.0 },
                PgPoint { x: 1.0, y: 1.0 },
                PgPoint { x: 1.0, y: 0.0 },
            ],
        }),
        // Box normalizes to (upper-right),(lower-left); supply already-normalized.
        bounds: Geo::new(PgBox {
            upper_right_x: 2.0,
            upper_right_y: 2.0,
            lower_left_x: 0.0,
            lower_left_y: 0.0,
        }),
        area: Geo::new(PgCircle {
            x: 1.0,
            y: 1.0,
            radius: 5.0,
        }),
        segment: Geo::new(PgLSeg {
            start_x: 0.0,
            start_y: 0.0,
            end_x: 3.0,
            end_y: 4.0,
        }),
        line: Geo::new(PgLine {
            a: 1.0,
            b: -1.0,
            c: 0.0,
        }),
        route: Geo::new(PgPath {
            closed: true,
            points: vec![
                PgPoint { x: 0.0, y: 0.0 },
                PgPoint { x: 1.0, y: 0.0 },
                PgPoint { x: 1.0, y: 1.0 },
            ],
        }),
        optional_point: Some(Geo::new(PgPoint { x: 9.0, y: 9.0 })),
    };

    let inserted = model.clone().into_active_model().insert(db).await?;
    assert_eq!(inserted, model);

    let fetched = GeoEntity::find_by_id(1).one(db).await?.expect("row exists");
    assert_eq!(fetched, model);
    // Deref lets us reach the sqlx type's fields directly.
    assert_eq!(fetched.location.x, 1.5);
    assert_eq!(fetched.location.y, 2.5);

    // NULL geometric round-trips.
    let model2 = geo::Model {
        id: 2,
        optional_point: None,
        ..model.clone()
    };
    let m2 = geo::ActiveModel {
        id: Set(2),
        optional_point: Set(None),
        ..model.clone().into_active_model()
    };
    m2.insert(db).await?;
    let fetched2 = GeoEntity::find_by_id(2).one(db).await?.expect("row exists");
    assert_eq!(fetched2.optional_point, None);
    let _ = model2;

    Ok(())
}

#[cfg(all(feature = "sqlx-postgres", feature = "postgres-geometry"))]
async fn spatial_query(db: &DatabaseConnection) -> Result<(), DbErr> {
    // Native geometric operator: `<->` planar distance, `@>` contains.
    // Which points lie within the boundary polygon of row 1?
    let stmt = Statement::from_string(
        DatabaseBackend::Postgres,
        r#"SELECT "id" FROM "geo" WHERE "boundary" @> point(0.5, 0.5) ORDER BY "id""#,
    );
    let rows = db.query_all_raw(stmt).await?;
    let ids: Vec<i32> = rows
        .iter()
        .map(|r| r.try_get::<i32>("", "id"))
        .collect::<Result<_, _>>()?;
    assert_eq!(ids, vec![1, 2], "(0.5,0.5) is inside both boundary polygons");

    // Planar distance from location to a probe point.
    let dist: f64 = db
        .query_one_raw(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"SELECT ("location" <-> point(4.5, 6.5)) AS d FROM "geo" WHERE "id" = 1"#,
        ))
        .await?
        .expect("row")
        .try_get::<f64>("", "d")?;
    assert_eq!(dist, 5.0, "distance (1.5,2.5)->(4.5,6.5) = 5");

    Ok(())
}
