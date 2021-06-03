use quote::*;
use sea_orm::EntityGenerator;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

/// Ref: https://stackoverflow.com/questions/65764987/how-to-pretty-print-syn-ast
fn write_and_fmt<P: AsRef<Path>, S: ToString>(path: P, code: S) -> io::Result<()> {
    fs::write(&path, code.to_string())?;

    Command::new("rustfmt")
        .arg(path.as_ref())
        .spawn()?
        .wait()?;

    Ok(())
}

fn test_fmt() {
    let mut code = quote!();
    #[allow(nonstandard_style)]
    for N in 0 .. 5_usize {
        code.extend(quote! {
            impl<Item> IsArray for [Item; #N] {
                type Item = Item;
                const LEN: usize = #N;
            }
        });
    }
    write_and_fmt("./test.rs", code).expect("unable to save or format");
}

#[async_std::main]
async fn main() {
    let uri = "mysql://sea:sea@localhost/bakery";
    let schema = "bakery";
    let generator = EntityGenerator::new(uri, schema)
        .await
        .parse();
}
