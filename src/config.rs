use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::Path;

pub const DEFAULT_CONFIG: &str = r#"env:
  PORT: "8080"
commands:
  - "go run main.go"
watch_dir: "./"
ignore:
  - "**/.git/**"
  - "**/target/**"
  - "**/node_modules/**"
  - "*.log"
  - "*.tmp"
"#;

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

pub fn init_config(path: &str, force: bool) -> io::Result<()> {
    let path = Path::new(path);
    if path.exists() && !force {
        return Err(io::Error::new(
            ErrorKind::AlreadyExists,
            format!("configuration file already exists: {}", path.display()),
        ));
    }

    fs::write(path, DEFAULT_CONFIG)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_config_path(name: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        std::env::temp_dir()
            .join(format!("watchx-{name}-{}-{nanos}.yaml", std::process::id()))
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn init_config_writes_default_config() {
        let path = temp_config_path("init");

        init_config(&path, false).expect("init config should write default config");

        let content = fs::read_to_string(&path).expect("default config should be readable");
        assert_eq!(content, DEFAULT_CONFIG);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn init_config_refuses_to_overwrite_existing_file_without_force() {
        let path = temp_config_path("exists");
        fs::write(&path, "existing: true\n").expect("existing config should be written");

        let error = init_config(&path, false).expect_err("init should not overwrite by default");

        assert_eq!(error.kind(), ErrorKind::AlreadyExists);
        assert_eq!(
            fs::read_to_string(&path).expect("existing config should remain readable"),
            "existing: true\n"
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn init_config_overwrites_existing_file_with_force() {
        let path = temp_config_path("force");
        fs::write(&path, "existing: true\n").expect("existing config should be written");

        init_config(&path, true).expect("force init should overwrite existing config");

        assert_eq!(
            fs::read_to_string(&path).expect("default config should be readable"),
            DEFAULT_CONFIG
        );

        let _ = fs::remove_file(path);
    }
}
