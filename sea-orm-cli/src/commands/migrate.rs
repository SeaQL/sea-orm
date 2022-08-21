use chrono::Local;
use regex::Regex;
use std::{error::Error, fs, io::Write, path::Path, process::Command};

use crate::MigrateSubcommands;

pub fn run_migrate_command(
    command: Option<MigrateSubcommands>,
    database_schema: &str,
    migration_dir: &str,
    verbose: bool,
) -> Result<(), Box<dyn Error>> {
    match command {
        Some(MigrateSubcommands::Init) => run_migrate_init(migration_dir)?,
        Some(MigrateSubcommands::Generate { migration_name }) => {
            run_migrate_generate(migration_dir, &migration_name)?
        }
        _ => {
            let (subcommand, database_schema, migration_dir, steps, verbose) = match command {
                Some(MigrateSubcommands::Fresh) => {
                    ("fresh", database_schema, migration_dir, None, verbose)
                }
                Some(MigrateSubcommands::Refresh) => {
                    ("refresh", database_schema, migration_dir, None, verbose)
                }
                Some(MigrateSubcommands::Reset) => {
                    ("reset", database_schema, migration_dir, None, verbose)
                }
                Some(MigrateSubcommands::Status) => {
                    ("status", database_schema, migration_dir, None, verbose)
                }
                Some(MigrateSubcommands::Up { num }) => {
                    ("up", database_schema, migration_dir, Some(num), verbose)
                }
                Some(MigrateSubcommands::Down { num }) => {
                    ("down", database_schema, migration_dir, Some(num), verbose)
                }
                _ => ("up", database_schema, migration_dir, None, verbose),
            };

            // Construct the `--manifest-path`
            let manifest_path = if migration_dir.ends_with('/') {
                format!("{}Cargo.toml", migration_dir)
            } else {
                format!("{}/Cargo.toml", migration_dir)
            };
            // Construct the arguments that will be supplied to `cargo` command
            let database_schema_option = &format!("--database-schema={}", database_schema);
            let mut args = vec![
                "run",
                "--manifest-path",
                manifest_path.as_str(),
                "--",
                database_schema_option,
                subcommand,
            ];

            let mut num: String = "".to_string();
            if let Some(steps) = steps {
                num = steps.to_string();
            }
            if !num.is_empty() {
                args.extend(["-n", num.as_str()])
            }
            if verbose {
                args.push("-v");
            }
            // Run migrator CLI on user's behalf
            println!("Running `cargo {}`", args.join(" "));
            Command::new("cargo").args(args).spawn()?.wait()?;
        }
    }

    Ok(())
}

pub fn run_migrate_init(migration_dir: &str) -> Result<(), Box<dyn Error>> {
    let migration_dir = match migration_dir.ends_with('/') {
        true => migration_dir.to_string(),
        false => format!("{}/", migration_dir),
    };
    println!("Initializing migration directory...");
    macro_rules! write_file {
        ($filename: literal) => {
            let fn_content = |content: String| content;
            write_file!($filename, $filename, fn_content);
        };
        ($filename: literal, $template: literal, $fn_content: expr) => {
            let filepath = [&migration_dir, $filename].join("");
            println!("Creating file `{}`", filepath);
            let path = Path::new(&filepath);
            let prefix = path.parent().unwrap();
            fs::create_dir_all(prefix).unwrap();
            let mut file = fs::File::create(path)?;
            let content = include_str!(concat!("../../template/migration/", $template));
            let content = $fn_content(content.to_string());
            file.write_all(content.as_bytes())?;
        };
    }
    write_file!("src/lib.rs");
    write_file!("src/m20220101_000001_create_table.rs");
    write_file!("src/main.rs");
    write_file!("Cargo.toml", "_Cargo.toml", |content: String| {
        let ver = format!(
            "^{}.{}.0",
            env!("CARGO_PKG_VERSION_MAJOR"),
            env!("CARGO_PKG_VERSION_MINOR")
        );
        content.replace("<sea-orm-migration-version>", &ver)
    });
    write_file!("README.md");
    println!("Done!");

    Ok(())
}

pub fn run_migrate_generate(
    migration_dir: &str,
    migration_name: &str,
) -> Result<(), Box<dyn Error>> {
    println!("Generating new migration...");

    // build new migration filename
    let now = Local::now();
    let migration_name = format!("m{}_{}", now.format("%Y%m%d_%H%M%S"), migration_name);

    create_new_migration(&migration_name, migration_dir)?;
    update_migrator(&migration_name, migration_dir)?;

    Ok(())
}

fn create_new_migration(migration_name: &str, migration_dir: &str) -> Result<(), Box<dyn Error>> {
    let migration_filepath = Path::new(migration_dir)
        .join("src")
        .join(format!("{}.rs", &migration_name));
    println!("Creating migration file `{}`", migration_filepath.display());
    // TODO: make OS agnostic
    let migration_template =
        include_str!("../../template/migration/src/m20220101_000001_create_table.rs");
    let mut migration_file = fs::File::create(migration_filepath)?;
    migration_file.write_all(migration_template.as_bytes())?;
    Ok(())
}

fn update_migrator(migration_name: &str, migration_dir: &str) -> Result<(), Box<dyn Error>> {
    let migrator_filepath = Path::new(migration_dir).join("src").join("lib.rs");
    println!(
        "Adding migration `{}` to `{}`",
        migration_name,
        migrator_filepath.display()
    );
    let migrator_content = fs::read_to_string(&migrator_filepath)?;
    let mut updated_migrator_content = migrator_content.clone();

    // create a backup of the migrator file in case something goes wrong
    let migrator_backup_filepath = migrator_filepath.with_file_name("lib.rs.bak");
    fs::copy(&migrator_filepath, &migrator_backup_filepath)?;
    let mut migrator_file = fs::File::create(&migrator_filepath)?;

    // find existing mod declarations, add new line
    let mod_regex = Regex::new(r"mod\s+(?P<name>m\d{8}_\d{6}_\w+);")?;
    let mods: Vec<_> = mod_regex.captures_iter(&migrator_content).collect();
    let mods_end = mods.last().unwrap().get(0).unwrap().end() + 1;
    updated_migrator_content.insert_str(mods_end, format!("mod {};\n", migration_name).as_str());

    // build new vector from declared migration modules
    let mut migrations: Vec<&str> = mods
        .iter()
        .map(|cap| cap.name("name").unwrap().as_str())
        .collect();
    migrations.push(migration_name);
    let mut boxed_migrations = migrations
        .iter()
        .map(|migration| format!("            Box::new({}::Migration),", migration))
        .collect::<Vec<String>>()
        .join("\n");
    boxed_migrations.push('\n');
    let boxed_migrations = format!("vec![\n{}        ]\n", boxed_migrations);
    let vec_regex = Regex::new(r"vec!\[[\s\S]+\]\n")?;
    let updated_migrator_content = vec_regex.replace(&updated_migrator_content, &boxed_migrations);

    migrator_file.write_all(updated_migrator_content.as_bytes())?;
    fs::remove_file(&migrator_backup_filepath)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_new_migration() {
        let migration_name = "test_name";
        let migration_dir = "/tmp/sea_orm_cli_test_new_migration/";
        fs::create_dir_all(format!("{}src", migration_dir)).unwrap();
        create_new_migration(migration_name, migration_dir).unwrap();
        let migration_filepath = Path::new(migration_dir)
            .join("src")
            .join(format!("{}.rs", migration_name));
        assert!(migration_filepath.exists());
        let migration_content = fs::read_to_string(migration_filepath).unwrap();
        assert_eq!(
            &migration_content,
            include_str!("../../template/migration/src/m20220101_000001_create_table.rs")
        );
        fs::remove_dir_all("/tmp/sea_orm_cli_test_new_migration/").unwrap();
    }

    #[test]
    fn test_update_migrator() {
        let migration_name = "test_name";
        let migration_dir = "/tmp/sea_orm_cli_test_update_migrator/";
        fs::create_dir_all(format!("{}src", migration_dir)).unwrap();
        let migrator_filepath = Path::new(migration_dir).join("src").join("lib.rs");
        fs::copy("./template/migration/src/lib.rs", &migrator_filepath).unwrap();
        update_migrator(migration_name, migration_dir).unwrap();
        assert!(&migrator_filepath.exists());
        let migrator_content = fs::read_to_string(&migrator_filepath).unwrap();
        let mod_regex = Regex::new(r"mod (?P<name>\w+);").unwrap();
        let migrations: Vec<&str> = mod_regex
            .captures_iter(&migrator_content)
            .map(|cap| cap.name("name").unwrap().as_str())
            .collect();
        assert_eq!(migrations.len(), 2);
        assert_eq!(
            *migrations.first().unwrap(),
            "m20220101_000001_create_table"
        );
        assert_eq!(migrations.last().unwrap(), &migration_name);
        let boxed_regex = Regex::new(r"Box::new\((?P<name>\S+)::Migration\)").unwrap();
        let migrations: Vec<&str> = boxed_regex
            .captures_iter(&migrator_content)
            .map(|cap| cap.name("name").unwrap().as_str())
            .collect();
        assert_eq!(migrations.len(), 2);
        assert_eq!(
            *migrations.first().unwrap(),
            "m20220101_000001_create_table"
        );
        assert_eq!(migrations.last().unwrap(), &migration_name);
        fs::remove_dir_all("/tmp/sea_orm_cli_test_update_migrator/").unwrap();
    }
}
