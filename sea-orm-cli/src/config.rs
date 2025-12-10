use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::{env, error::Error, fs};

#[derive(Deserialize)]
pub struct Config {
    database: Database,
    migrations: Migrations,
}

#[derive(Deserialize)]
struct Database {
    url: String,
}

#[derive(Deserialize)]
struct Migrations {
    directory: String,
}

/// The config file is expected to be in the current directory or a parent directory
pub fn get_config() -> Result<Config, Box<dyn Error>> {
    let config_path = find_config_file()?;

    let file_content = fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&file_content)?;
    Ok(config)
}

pub fn get_database_url() -> Result<String, Box<dyn Error>> {
    let config = get_config()?;
    if config.database.url.starts_with("env:") {
        let env_var = config.database.url.split(":").nth(1).unwrap();
        let value = env::var(env_var)?;
        Ok(value)
    } else {
        Ok(config.database.url)
    }
}

pub fn get_migration_dir() -> Result<String, Box<dyn Error>> {
    let config = get_config()?;

    let migration_dir = config.migrations.directory;
    if migration_dir.starts_with(".") {
        let config_dir = get_config_dir()?;
        let migration_dir = config_dir.join(migration_dir);
        return Ok(migration_dir.to_string_lossy().to_string());
    } else {
        return Ok(migration_dir);
    }
}

fn get_config_dir() -> Result<PathBuf, Box<dyn Error>> {
    let config_path = find_config_file()?;
    let config_dir = config_path.parent().unwrap_or(Path::new(".")).to_path_buf();
    Ok(config_dir)
}

fn find_config_file() -> Result<PathBuf, Box<dyn Error>> {
    let current_dir = std::env::current_dir()?;
    let config_path = current_dir.join("sea-orm.toml");
    if config_path.exists() {
        return Ok(config_path);
    }
    let parent_dir = current_dir.parent();
    if let Some(parent_dir) = parent_dir {
        let config_path = parent_dir.join("sea-orm.toml");
        if config_path.exists() {
            return Ok(config_path);
        }
    }
    Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "SeaORM config file not found, use `sea-orm-cli config init` to create one",
    )))
}
