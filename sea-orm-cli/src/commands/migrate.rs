use chrono::{Local, Utc};
use regex::Regex;
use std::{
    error::Error,
    fmt::Display,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

#[cfg(feature = "cli")]
use crate::MigrateSubcommands;

#[cfg(feature = "cli")]
pub fn run_migrate_command(
    command: Option<MigrateSubcommands>,
    migration_dir: &str,
    database_schema: Option<String>,
    database_url: Option<String>,
    verbose: bool,
) -> Result<(), Box<dyn Error>> {
    match command {
        Some(MigrateSubcommands::Init) => run_migrate_init(migration_dir)?,
        Some(MigrateSubcommands::Generate {
            migration_name,
            universal_time: _,
            local_time,
        }) => run_migrate_generate(migration_dir, &migration_name, !local_time)?,
        _ => {
            let (subcommand, migration_dir, steps, verbose) = match command {
                Some(MigrateSubcommands::Fresh) => ("fresh", migration_dir, None, verbose),
                Some(MigrateSubcommands::Refresh) => ("refresh", migration_dir, None, verbose),
                Some(MigrateSubcommands::Reset) => ("reset", migration_dir, None, verbose),
                Some(MigrateSubcommands::Status) => ("status", migration_dir, None, verbose),
                Some(MigrateSubcommands::Up { num }) => ("up", migration_dir, num, verbose),
                Some(MigrateSubcommands::Down { num }) => {
                    ("down", migration_dir, Some(num), verbose)
                }
                _ => ("up", migration_dir, None, verbose),
            };

            // Construct the `--manifest-path`
            let manifest_path = if migration_dir.ends_with('/') {
                format!("{migration_dir}Cargo.toml")
            } else {
                format!("{migration_dir}/Cargo.toml")
            };
            // Construct the arguments that will be supplied to `cargo` command
            let mut args = vec!["run", "--manifest-path", &manifest_path, "--", subcommand];

            let mut num: String = "".to_string();
            if let Some(steps) = steps {
                num = steps.to_string();
            }
            if !num.is_empty() {
                args.extend(["-n", &num])
            }
            if let Some(database_url) = &database_url {
                args.extend(["-u", database_url]);
            }
            if let Some(database_schema) = &database_schema {
                args.extend(["-s", database_schema]);
            }
            if verbose {
                args.push("-v");
            }
            // Run migrator CLI on user's behalf
            println!("Running `cargo {}`", args.join(" "));
            let exit_status = Command::new("cargo").args(args).status()?; // Get the status code
            if !exit_status.success() {
                // Propagate the error if any
                return Err("Fail to run migration".into());
            }
        }
    }

    Ok(())
}

pub fn run_migrate_init(migration_dir: &str) -> Result<(), Box<dyn Error>> {
    let migration_dir = match migration_dir.ends_with('/') {
        true => migration_dir.to_string(),
        false => format!("{migration_dir}/"),
    };
    println!("Initializing migration directory...");
    macro_rules! write_file {
        ($filename: literal) => {
            let fn_content = |content: String| content;
            write_file!($filename, $filename, fn_content);
        };
        ($filename: literal, $template: literal) => {
            let fn_content = |content: String| content;
            write_file!($filename, $template, fn_content);
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
            "{}.{}.0",
            env!("CARGO_PKG_VERSION_MAJOR"),
            env!("CARGO_PKG_VERSION_MINOR")
        );
        content.replace("<sea-orm-migration-version>", &ver)
    });
    write_file!("README.md");
    if git2::Repository::discover(Path::new(&migration_dir)).is_ok() {
        write_file!(".gitignore", "_gitignore");
    }
    println!("Done!");

    Ok(())
}

pub fn run_migrate_generate(
    migration_dir: &str,
    migration_name: &str,
    universal_time: bool,
) -> Result<(), Box<dyn Error>> {
    // Make sure the migration name doesn't contain any characters that
    // are invalid module names in Rust.
    if migration_name.contains('-') {
        return Err(Box::new(MigrationCommandError::InvalidName(
            "Hyphen `-` cannot be used in migration name".to_string(),
        )));
    }

    println!("Generating new migration...");

    // build new migration filename
    const FMT: &str = "%Y%m%d_%H%M%S";
    let formatted_now = if universal_time {
        Utc::now().format(FMT)
    } else {
        Local::now().format(FMT)
    };

    let migration_name = migration_name.trim().replace(' ', "_");
    let migration_name = format!("m{formatted_now}_{migration_name}");

    create_new_migration(&migration_name, migration_dir)?;
    update_migrator(&migration_name, migration_dir)?;

    Ok(())
}

/// `get_full_migration_dir` looks for a `src` directory
/// inside of `migration_dir` and appends that to the returned path if found.
///
/// Otherwise, `migration_dir` can point directly to a directory containing the
/// migrations. In that case, nothing is appended.
///
/// This way, `src` doesn't need to be appended in the standard case where
/// migrations are in their own crate. If the migrations are in a submodule
/// of another crate, `migration_dir` can point directly to that module.
fn get_full_migration_dir(migration_dir: &str) -> PathBuf {
    let without_src = Path::new(migration_dir).to_owned();
    let with_src = without_src.join("src");
    match () {
        _ if with_src.is_dir() => with_src,
        _ => without_src,
    }
}

fn create_new_migration(migration_name: &str, migration_dir: &str) -> Result<(), Box<dyn Error>> {
    let migration_filepath =
        get_full_migration_dir(migration_dir).join(format!("{}.rs", &migration_name));
    println!("Creating migration file `{}`", migration_filepath.display());
    // TODO: make OS agnostic
    let migration_template =
        include_str!("../../template/migration/src/m20220101_000001_create_table.rs");
    let mut migration_file = fs::File::create(migration_filepath)?;
    migration_file.write_all(migration_template.as_bytes())?;
    Ok(())
}

/// `get_migrator_filepath` looks for a file `migration_dir/src/lib.rs`
/// and returns that path if found.
///
/// If `src` is not found, it will look directly in `migration_dir` for `lib.rs`.
///
/// If `lib.rs` is not found, it will look for `mod.rs` instead,
/// e.g. `migration_dir/mod.rs`.
///
/// This way, `src` doesn't need to be appended in the standard case where
/// migrations are in their own crate (with a file `lib.rs`). If the
/// migrations are in a submodule of another crate (with a file `mod.rs`),
/// `migration_dir` can point directly to that module.
fn get_migrator_filepath(migration_dir: &str) -> PathBuf {
    let full_migration_dir = get_full_migration_dir(migration_dir);
    let with_lib = full_migration_dir.join("lib.rs");
    match () {
        _ if with_lib.is_file() => with_lib,
        _ => full_migration_dir.join("mod.rs"),
    }
}

fn update_migrator(migration_name: &str, migration_dir: &str) -> Result<(), Box<dyn Error>> {
    let migrator_filepath = get_migrator_filepath(migration_dir);
    println!(
        "Adding migration `{}` to `{}`",
        migration_name,
        migrator_filepath.display()
    );
    let migrator_content = fs::read_to_string(&migrator_filepath)?;
    let mut updated_migrator_content = migrator_content.clone();

    // create a backup of the migrator file in case something goes wrong
    let migrator_backup_filepath = migrator_filepath.with_extension("rs.bak");
    fs::copy(&migrator_filepath, &migrator_backup_filepath)?;
    let mut migrator_file = fs::File::create(&migrator_filepath)?;

    // find existing mod declarations, add new line
    let mod_regex = Regex::new(r"mod\s+(?P<name>m\d{8}_\d{6}_\w+);")?;
    let mods: Vec<_> = mod_regex.captures_iter(&migrator_content).collect();
    let mods_end = mods.last().unwrap().get(0).unwrap().end() + 1;
    updated_migrator_content.insert_str(mods_end, format!("mod {migration_name};\n").as_str());

    // build new vector from declared migration modules
    let mut migrations: Vec<&str> = mods
        .iter()
        .map(|cap| cap.name("name").unwrap().as_str())
        .collect();
    migrations.push(migration_name);
    let mut boxed_migrations = migrations
        .iter()
        .map(|migration| format!("            Box::new({migration}::Migration),"))
        .collect::<Vec<String>>()
        .join("\n");
    boxed_migrations.push('\n');
    let boxed_migrations = format!("vec![\n{boxed_migrations}        ]\n");
    let vec_regex = Regex::new(r"vec!\[[\s\S]+\]\n")?;
    let updated_migrator_content = vec_regex.replace(&updated_migrator_content, &boxed_migrations);

    migrator_file.write_all(updated_migrator_content.as_bytes())?;
    fs::remove_file(&migrator_backup_filepath)?;
    Ok(())
}

#[derive(Debug)]
enum MigrationCommandError {
    InvalidName(String),
}

impl Display for MigrationCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MigrationCommandError::InvalidName(name) => {
                write!(f, "Invalid migration name: {name}")
            }
        }
    }
}

impl Error for MigrationCommandError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_new_migration() {
        let migration_name = "test_name";
        let migration_dir = "/tmp/sea_orm_cli_test_new_migration/";
        fs::create_dir_all(format!("{migration_dir}src")).unwrap();
        create_new_migration(migration_name, migration_dir).unwrap();
        let migration_filepath = Path::new(migration_dir)
            .join("src")
            .join(format!("{migration_name}.rs"));
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
        fs::create_dir_all(format!("{migration_dir}src")).unwrap();
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
