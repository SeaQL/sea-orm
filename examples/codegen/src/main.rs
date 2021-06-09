use sea_orm_codegen::{EntityGenerator, Error};

#[async_std::main]
async fn main() -> Result<(), Error> {
    let uri = "mysql://sea:sea@localhost/bakery";
    let schema = "bakery";

    let _generator = EntityGenerator::discover(uri, schema).await?
        .transform()?
        .generate("out")?;

    Ok(())
}
