use sea_orm_codegen::EntityGenerator;

#[async_std::main]
async fn main() {
    let uri = "mysql://sea:sea@localhost/bakery";
    let schema = "bakery";
    let _generator = EntityGenerator::new()
        .parse(uri, schema).await
        .write("out").unwrap();
}
