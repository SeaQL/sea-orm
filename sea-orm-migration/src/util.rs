use handlebars::Handlebars;
use serde_json::json;

fn migration_name_prefix() -> String{
    format!("m{}", Local::now().format("%Y%m%d_%H%M%S").to_string())
}

pub fn get_file_stem(path: &str) -> &str {
    let datatime = migration_name_prefix();
    // replace path
    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("path", &path).unwrap();
    let path = handlebars.render("path", &json!({"datatime": datatime})).unwrap();
    std::path::Path::new(path)
        .file_stem()
        .map(|f| f.to_str().unwrap())
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
        ];
        for (path, expect) in pair {
            assert_eq!(get_file_stem(path), expect);
        }
    }
}
