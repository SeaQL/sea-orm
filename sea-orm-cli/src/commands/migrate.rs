use chrono::{Local, Utc};
use colored::Colorize;
use regex::Regex;
use std::{
    error::Error,
    fmt::Display,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[cfg(feature = "cli")]
use crate::MigrateSubcommands;
use crate::commands::subprocess::{
    AppliedData, LifecycleData, RolledBackData, StatusData, SubprocessError, manifest_path,
    run_subprocess_json,
};

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
        cmd => run_migrate_json(
            cmd,
            migration_dir,
            database_url.as_deref(),
            database_schema.as_deref(),
            verbose,
        )?,
    }

    Ok(())
}

fn run_migrate_json(
    command: Option<MigrateSubcommands>,
    migration_dir: &str,
    database_url: Option<&str>,
    database_schema: Option<&str>,
    _verbose: bool,
) -> Result<(), Box<dyn Error>> {
    let manifest = manifest_path(migration_dir);

    match command {
        Some(MigrateSubcommands::Status) | None => {
            match run_subprocess_json::<StatusData>(
                &manifest,
                &["status"],
                database_url,
                database_schema,
            ) {
                Ok((_, data)) => {
                    print_status(&data);
                    return Ok(());
                }
                Err(e) => return Err(render_subprocess_error(e).into()),
            }
        }
        Some(MigrateSubcommands::Up { num }) => {
            let mut args = vec!["up".to_string()];
            if let Some(n) = num {
                args.push(format!("-n={n}"));
            }
            let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();
            match run_subprocess_json::<AppliedData>(
                &manifest,
                &args_ref,
                database_url,
                database_schema,
            ) {
                Ok((_, data)) => {
                    print_applied(&data);
                    return Ok(());
                }
                Err(e) => return Err(render_subprocess_error(e).into()),
            }
        }
        Some(MigrateSubcommands::Down { num }) => {
            let n_str = num.to_string();
            let args = ["down", "-n", &n_str];
            match run_subprocess_json::<RolledBackData>(
                &manifest,
                &args,
                database_url,
                database_schema,
            ) {
                Ok((_, data)) => {
                    print_rolled_back(&data);
                    return Ok(());
                }
                Err(e) => return Err(render_subprocess_error(e).into()),
            }
        }
        Some(MigrateSubcommands::Fresh) => {
            match run_subprocess_json::<AppliedData>(
                &manifest,
                &["fresh"],
                database_url,
                database_schema,
            ) {
                Ok((_, data)) => {
                    print_applied(&data);
                    return Ok(());
                }
                Err(e) => return Err(render_subprocess_error(e).into()),
            }
        }
        Some(MigrateSubcommands::Refresh) => {
            match run_subprocess_json::<LifecycleData>(
                &manifest,
                &["refresh"],
                database_url,
                database_schema,
            ) {
                Ok((_, data)) => {
                    print_lifecycle(&data);
                    return Ok(());
                }
                Err(e) => return Err(render_subprocess_error(e).into()),
            }
        }
        Some(MigrateSubcommands::Reset) => {
            match run_subprocess_json::<RolledBackData>(
                &manifest,
                &["reset"],
                database_url,
                database_schema,
            ) {
                Ok((_, data)) => {
                    print_rolled_back(&data);
                    return Ok(());
                }
                Err(e) => return Err(render_subprocess_error(e).into()),
            }
        }
        Some(MigrateSubcommands::Init) | Some(MigrateSubcommands::Generate { .. }) => {
            unreachable!("init/generate handled before reaching this function")
        }
    }
}

fn render_subprocess_error(e: SubprocessError) -> String {
    match e {
        SubprocessError::VersionMismatch { expected, got } => {
            format!(
                "Version mismatch: CLI is {expected} but crate returned {got}.\n  \
                 Rebuild your migration crate with a matching sea-orm-migration version."
            )
        }
        other => other.to_string(),
    }
}

fn print_status(data: &StatusData) {
    println!();
    if data.migrations.is_empty() {
        println!("  No migrations found.");
    } else {
        let name_w = data
            .migrations
            .iter()
            .map(|m| m.name.len())
            .max()
            .unwrap_or(10);
        println!(
            "  {}",
            format!("{:<width$}  Status", "Migration", width = name_w).bold()
        );
        println!("  {}", "-".repeat(name_w + 12));
        for m in &data.migrations {
            let status_str = if m.status == "Applied" {
                "✓ Applied".green()
            } else {
                "○ Pending".yellow()
            };
            println!("  {:<width$}  {status_str}", m.name, width = name_w);
        }
    }
    println!();
}

fn print_applied(data: &AppliedData) {
    println!();
    if data.applied.is_empty() {
        println!("  {} No pending migrations.", "✓".green());
    } else {
        println!(
            "  {} Applied {} migration(s):",
            "✓".green(),
            data.applied.len()
        );
        for name in &data.applied {
            println!("      {} {name}", "+".green());
        }
    }
    println!();
}

fn print_rolled_back(data: &RolledBackData) {
    println!();
    if data.rolled_back.is_empty() {
        println!("  {} No applied migrations to roll back.", "○".yellow());
    } else {
        println!(
            "  {} Rolled back {} migration(s):",
            "↩".yellow(),
            data.rolled_back.len()
        );
        for name in &data.rolled_back {
            println!("      {} {name}", "-".yellow());
        }
    }
    println!();
}

fn print_lifecycle(data: &LifecycleData) {
    println!();
    if !data.rolled_back.is_empty() {
        println!(
            "  {} Rolled back {} migration(s):",
            "↩".yellow(),
            data.rolled_back.len()
        );
        for name in &data.rolled_back {
            println!("      {} {name}", "-".yellow());
        }
    }
    if !data.applied.is_empty() {
        println!(
            "  {} Applied {} migration(s):",
            "✓".green(),
            data.applied.len()
        );
        for name in &data.applied {
            println!("      {} {name}", "+".green());
        }
    }
    if data.rolled_back.is_empty() && data.applied.is_empty() {
        println!("  {} Nothing to do.", "✓".green());
    }
    println!();
}

// ---------------------------------------------------------------------------
// Local scaffold helpers (unchanged from before, no JSON API needed)
// ---------------------------------------------------------------------------

pub fn run_migrate_init(migration_dir: &str) -> Result<(), Box<dyn Error>> {
    let migration_dir = match migration_dir.ends_with('/') {
        true => migration_dir.to_string(),
        false => format!("{migration_dir}/"),
    };
    println!("{}", "Initializing migration directory...".cyan());
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
            println!("Creating file `{}`", filepath.dimmed());
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
    if glob::glob(&format!("{migration_dir}**/.git"))?.count() > 0 {
        write_file!(".gitignore", "_gitignore");
    }
    println!("{}", "Done!".green().bold());

    Ok(())
}

pub fn run_migrate_generate(
    migration_dir: &str,
    migration_name: &str,
    universal_time: bool,
) -> Result<(), Box<dyn Error>> {
    if migration_name.contains('-') {
        return Err(Box::new(MigrationCommandError::InvalidName(
            "Hyphen `-` cannot be used in migration name".to_string(),
        )));
    }

    println!("{}", "Generating new migration...".cyan());

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

fn get_full_migration_dir(migration_dir: &str) -> PathBuf {
    let without_src = Path::new(migration_dir).to_owned();
    let with_src = without_src.join("src");
    match () {
        _ if with_src.is_dir() => with_src,
        _ => without_src,
    }
}

fn get_migrator_filepath(migration_dir: &str) -> PathBuf {
    let full_migration_dir = get_full_migration_dir(migration_dir);
    let with_lib = full_migration_dir.join("lib.rs");
    match () {
        _ if with_lib.is_file() => with_lib,
        _ => full_migration_dir.join("mod.rs"),
    }
}

fn create_new_migration(migration_name: &str, migration_dir: &str) -> Result<(), Box<dyn Error>> {
    let migration_filepath =
        get_full_migration_dir(migration_dir).join(format!("{}.rs", &migration_name));
    println!(
        "Creating migration file `{}`",
        migration_filepath.display().to_string().dimmed()
    );
    let migration_template =
        include_str!("../../template/migration/src/m20220101_000001_create_table.rs");
    let mut migration_file = fs::File::create(migration_filepath)?;
    migration_file.write_all(migration_template.as_bytes())?;
    Ok(())
}

fn update_migrator(migration_name: &str, migration_dir: &str) -> Result<(), Box<dyn Error>> {
    let migrator_filepath = get_migrator_filepath(migration_dir);
    println!(
        "Adding migration `{}` to `{}`",
        migration_name.cyan(),
        migrator_filepath.display().to_string().dimmed()
    );
    let migrator_content = fs::read_to_string(&migrator_filepath)?;
    let mut updated_migrator_content = migrator_content.clone();

    let migrator_backup_filepath = migrator_filepath.with_extension("rs.bak");
    fs::copy(&migrator_filepath, &migrator_backup_filepath)?;
    let mut migrator_file = fs::File::create(&migrator_filepath)?;

    let mod_regex = Regex::new(r"mod\s+(?P<name>m\d{8}_\d{6}_\w+);")?;
    let mods: Vec<_> = mod_regex.captures_iter(&migrator_content).collect();
    let mods_end = if let Some(last_match) = mods.last() {
        last_match.get(0).unwrap().end() + 1
    } else {
        migrator_content.len()
    };
    updated_migrator_content.insert_str(mods_end, format!("mod {migration_name};\n").as_str());

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
