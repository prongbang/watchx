use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub env: HashMap<String, String>,
    pub commands: Vec<String>,
    pub watch_dir: String,
    pub ignore: Option<Vec<String>>,
}

// Read and parse the configuration file
pub fn read_config(path: &str) -> Config {
    let config_content = fs::read_to_string(path).expect("Failed to read configuration file");
    serde_yaml::from_str(&config_content).expect("Failed to parse YAML configuration")
}