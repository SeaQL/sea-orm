use std::{error::Error, fs::File, io::Read};

pub fn parse_config<T: serde::de::DeserializeOwned + std::fmt::Debug>(
    config: String,
) -> Result<T, Box<dyn Error>> {
    let mut file = File::open(config)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let parsed_config: T = serde_json::from_str(&content)?;
    Ok(parsed_config)
}
