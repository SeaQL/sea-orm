#[cfg(test)]
mod test {
    #[test]
    fn test_entity_registry() {
        let entities: Vec<_> = inventory::iter::<crate::EntityRegistry>().collect();

        for target in [
            "sea_orm::tests_cfg::cake",
            "sea_orm::tests_cfg::cake_compact",
            "sea_orm::tests_cfg::cake_expanded",
            "sea_orm::tests_cfg::cake_filling",
            "sea_orm::tests_cfg::cake_filling_price",
            "sea_orm::tests_cfg::filling",
            "sea_orm::tests_cfg::fruit",
            "sea_orm::tests_cfg::indexes",
            "sea_orm::tests_cfg::ingredient",
            "sea_orm::tests_cfg::lunch_set",
            "sea_orm::tests_cfg::lunch_set_expanded",
            "sea_orm::tests_cfg::rust_keyword",
            "sea_orm::tests_cfg::vendor",
        ] {
            if !entities.iter().any(|e| e.module_path == target) {
                panic!("{target} not found");
            }
        }
    }
}
