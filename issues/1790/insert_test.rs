mod tests {
    // currently ok
    #[test]
    fn insert_do_nothing_postgres() {
        assert_eq!(
            Insert::<cake::ActiveModel>::new()
                .add(cake::Model {
                    id: 1,
                    name: "Apple Pie".to_owned(),
                })
                .on_conflict(OnConflict::new()
                    .do_nothing()
                    .to_owned()
                )
                .build(DbBackend::Postgres)
                .to_string(),
            r#"INSERT INTO "cake" ("id", "name") VALUES (1, 'Apple Pie') ON CONFLICT DO NOTHING"#,
        );
    }

    //failed to run
    #[test]
    fn insert_do_nothing_mysql() {
        assert_eq!(
            Insert::<cake::ActiveModel>::new()
                .add(cake::Model {
                    id: 1,
                    name: "Apple Pie".to_owned(),
                })
                .on_conflict(OnConflict::new()
                    .do_nothing()
                    .to_owned()
                )
                .build(DbBackend::Mysql)
                .to_string(),
            r#"INSERT IGNORE INTO "cake" ("id", "name") VALUES (1, 'Apple Pie')"#,
        );
    }

    // currently ok
    #[test]
    fn insert_do_nothing() {
        assert_eq!(
            Insert::<cake::ActiveModel>::new()
                .add(cake::Model {
                    id: 1,
                    name: "Apple Pie".to_owned(),
                })
                .on_conflict(OnConflict::new()
                    .do_nothing()
                    .to_owned()
                )
                .build(DbBackend::Sqlite)
                .to_string(),
            r#"INSERT IGNORE INTO "cake" ("id", "name") VALUES (1, 'Apple Pie')"#,
        );
    }
}