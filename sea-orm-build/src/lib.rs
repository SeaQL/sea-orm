use std::env;
use std::path::Path;

use error::Error;
use sea_orm_codegen::EntityTransformer;
use sea_schema::sea_query::TableStatement;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

mod error;

/// Generate models from a given mysql connection and tables.
///
/// Models can be imported in your app with the `sea_orm::include_model!("model")` macro.
#[cfg(feature = "mysql")]
pub async fn generate_models(url: &str, tables: &[&str]) -> Result<(), Error> {
    use sea_schema::mysql::discovery::SchemaDiscovery;
    use sqlx::MySqlPool;

    let url_parts: Vec<&str> = url.split("/").collect();
    let schema = url_parts.last().unwrap();
    let connection = MySqlPool::connect(url).await.map_err(Error::Sqlx)?;
    let schema_discovery = SchemaDiscovery::new(connection, schema);
    let table_stmts = schema
        .tables
        .into_iter()
        .filter(|schema| tables.contains(&schema.info.name))
        .map(|schema| schema.write())
        .collect();

    write_table_stmts(table_stmts).await
}

/// Generate models from a given postgres connection and tables.
///
/// Models can be imported in your app with the `sea_orm::include_model!("model")` macro.
#[cfg(feature = "postgres")]
pub async fn generate_models(schema: &str, url: &str, tables: &[&str]) -> Result<(), Error> {
    use sea_schema::postgres::discovery::SchemaDiscovery;
    use sqlx::PgPool;

    let connection = PgPool::connect(url).await.map_err(Error::Sqlx)?;
    let schema_discovery = SchemaDiscovery::new(connection, schema);
    let schema = schema_discovery.discover().await;
    let table_stmts = schema
        .tables
        .into_iter()
        .filter(|schema| tables.contains(&schema.info.name.as_str()))
        .map(|schema| schema.write())
        .collect();

    write_table_stmts(table_stmts).await
}

async fn write_table_stmts(table_stmts: Vec<TableStatement>) -> Result<(), Error> {
    let output = EntityTransformer::transform(table_stmts)
        .map_err(Error::SeaOrmCodegen)?
        .generate(false, false);

    let out_dir_string = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir_string);

    for output_file in output.files.iter() {
        let file_path = out_dir.join(&output_file.name);
        let mut file = File::create(file_path).await.map_err(Error::Io)?;
        file.write_all(output_file.content.as_bytes())
            .await
            .map_err(Error::Io)?;
    }

    for output_file in output.files.iter() {
        Command::new("rustfmt")
            .arg(out_dir.join(&output_file.name))
            .spawn()
            .map_err(Error::Io)?
            .wait()
            .await
            .map_err(Error::Io)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_1() {
        #[cfg(feature = "postgres")]
        crate::generate_models(
            "public",
            "postgres://ari@0.0.0.0:5432/product",
            &["product", "variant"],
        )
        .await
        .expect("could not generate models");
    }
}
