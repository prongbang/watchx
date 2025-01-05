use notify::{Watcher, RecommendedWatcher, RecursiveMode, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::{Command, Child};
use std::sync::mpsc::channel;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct Config {
    env: HashMap<String, String>,  // Environment variables
    commands: Vec<String>,         // List of commands to execute
    watch_dir: String,             // Directory to watch
}

// Read and parse the configuration file
fn read_config(path: &str) -> Config {
    let config_content = fs::read_to_string(path).expect("Failed to read configuration file");
    serde_yaml::from_str(&config_content).expect("Failed to parse YAML configuration")
}

// Execute a list of commands in sequence
fn execute_commands(commands: &[String], env: &HashMap<String, String>) -> Vec<Child> {
    let mut children = Vec::new();

    for command in commands {
        let parts: Vec<&str> = command.split_whitespace().collect();
        let (cmd, args) = parts.split_first().expect("Invalid command");

        let child = Command::new(cmd)
            .args(args)
            .envs(env.clone()) // Set environment variables
            .spawn()
            .expect("Failed to execute command");

        children.push(child);
    }

    children
}

fn main() -> Result<()> {
    // Load configuration
    let config = read_config("reloadx.yaml");

    // Channel to receive file change events
    let (tx, rx) = channel();

    // Set up file watcher
    let watch_config = notify::Config::default()
        .with_poll_interval(Duration::from_secs(2))
        .with_compare_contents(true);
    let mut watcher: RecommendedWatcher = Watcher::new(tx, watch_config)?;
    let path = Path::new(&config.watch_dir);
    watcher.watch(path, RecursiveMode::Recursive)?;

    // Execute initial commands
    let mut children = execute_commands(&config.commands, &config.env);

    for res in rx {
        match res {
            Ok(event) => {
                println!("Change detected: {:?}", event);

                // Kill all running processes
                for child in &mut children {
                    child.kill().expect("Failed to kill process");
                }

                // Restart the commands
                children = execute_commands(&config.commands, &config.env);
            }
            Err(e) => println!("Watch error: {:?}", e),
        }
    }

    Ok(())
}
