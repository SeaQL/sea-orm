use regex::Regex;
use sea_orm::Statement;
use std::{error::Error, fs, path::PathBuf};

use crate::codegen::{MigrationMetadata, render_migration_file};

pub fn write_migration(
    migration_dir: &str,
    migration_name: &str,
    stmts: &[Statement],
    meta: &MigrationMetadata<'_>,
) -> Result<PathBuf, Box<dyn Error>> {
    let filepath = write_migration_file(migration_dir, migration_name, stmts, meta)?;
    update_migrator(migration_dir, migration_name)?;
    Ok(filepath)
}

fn get_full_migration_dir(migration_dir: &str) -> PathBuf {
    let base = PathBuf::from(migration_dir);
    let with_src = base.join("src");
    if with_src.is_dir() { with_src } else { base }
}

fn get_migrator_filepath(migration_dir: &str) -> PathBuf {
    let full_dir = get_full_migration_dir(migration_dir);
    let with_lib = full_dir.join("lib.rs");
    if with_lib.is_file() {
        with_lib
    } else {
        full_dir.join("mod.rs")
    }
}

fn write_migration_file(
    migration_dir: &str,
    migration_name: &str,
    stmts: &[Statement],
    meta: &MigrationMetadata<'_>,
) -> Result<PathBuf, Box<dyn Error>> {
    let filepath = get_full_migration_dir(migration_dir).join(format!("{migration_name}.rs"));
    println!("Creating migration file `{}`", filepath.display());
    let content = render_migration_file(stmts, meta);
    fs::write(&filepath, content.as_bytes())?;
    Ok(filepath)
}

fn update_migrator(migration_dir: &str, migration_name: &str) -> Result<(), Box<dyn Error>> {
    let migrator_filepath = get_migrator_filepath(migration_dir);
    println!(
        "Adding migration `{migration_name}` to `{}`",
        migrator_filepath.display()
    );
    let original = fs::read_to_string(&migrator_filepath)?;

    // Find existing mod declarations and get insertion index for a new one
    let mod_regex = Regex::new(r"mod\s+(?P<name>m\d{8}_\d{6}_\w+);")?;
    let mods: Vec<_> = mod_regex.captures_iter(&original).collect();
    let insert_pos = if let Some(last_match) = mods.last() {
        last_match.get(0).unwrap().end() + 1
    } else {
        // Insert at the beginning of the file (before `pub struct Migrator`)
        original.find("pub struct").unwrap_or(original.len())
    };

    // Insert the new mod declaration.
    let new_mod_decl = if mods.is_empty() {
        //When inserting before the struct, add a blank line to look nicer
        format!("mod {migration_name};\n\n")
    } else {
        format!("mod {migration_name};\n")
    };
    let mut updated = original.clone();
    updated.insert_str(insert_pos, &new_mod_decl);

    // Rebuild the migrations vec
    let mut migrations: Vec<&str> = mods
        .iter()
        .map(|cap| cap.name("name").unwrap().as_str())
        .collect();
    migrations.push(migration_name);
    let boxed = migrations
        .iter()
        .map(|m| format!("            Box::new({m}::Migration),"))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    let new_vec = format!("vec![\n{boxed}        ]");

    // Match both empty vec![] and vec![...]
    let vec_regex = Regex::new(r"vec!\[[\s\S]*?\]")?;
    let updated = vec_regex.replace(&updated, new_vec.as_str());

    // write to a temp file beside the target, then rename
    let tmp_path = migrator_filepath.with_extension("rs.tmp");
    fs::write(&tmp_path, updated.as_bytes())?;
    fs::rename(&tmp_path, &migrator_filepath)?;
    Ok(())
}
