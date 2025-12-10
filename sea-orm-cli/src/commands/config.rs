use std::path::Path;
use std::{error::Error, fs};

use crate::ConfigSubcommands;

pub fn run_config_command(command: ConfigSubcommands) -> Result<(), Box<dyn Error>> {
    match command {
        ConfigSubcommands::Init => run_config_init(),
    }
}

fn run_config_init() -> Result<(), Box<dyn Error>> {
    let config_path = Path::new("sea-orm.toml");
    let config_template = include_str!("../../template/sea-orm.toml");

    fs::write(config_path, config_template.to_string())?;

    println!("Config file created at {}", config_path.display());
    Ok(())
}
