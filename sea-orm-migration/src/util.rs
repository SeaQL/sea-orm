pub fn get_file_stem(path: &str) -> &str {
    let path = std::path::Path::new(path);
    let file_name = path.file_name().and_then(|f| f.to_str()).unwrap();

    if file_name == "mod.rs" {
        path.parent()
            .and_then(|p| p.file_name())
            .and_then(|f| f.to_str())
    } else {
        path.file_stem().and_then(|f| f.to_str())
    }
    .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_file_stem() {
        let pair = vec![
            (
                "m20220101_000001_create_table.rs",
                "m20220101_000001_create_table",
            ),
            (
                "src/m20220101_000001_create_table.rs",
                "m20220101_000001_create_table",
            ),
            (
                "migration/src/m20220101_000001_create_table.rs",
                "m20220101_000001_create_table",
            ),
            (
                "/migration/src/m20220101_000001_create_table.tmp.rs",
                "m20220101_000001_create_table.tmp",
            ),
            (
                "migration/src/m20220101_000001_create_table/mod.rs",
                "m20220101_000001_create_table",
            ),
        ];
        for (path, expect) in pair {
            assert_eq!(get_file_stem(path), expect);
        }
    }
}
