use notify::{RecommendedWatcher, RecursiveMode, Result, Watcher};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::str::FromStr;
use std::sync::{mpsc::channel, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Deserialize)]
struct Config {
    env: HashMap<String, String>,
    commands: Vec<String>,
    watch_dir: String,
    ignore: Option<Vec<String>>,
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
            .envs(env.clone())
            .spawn()
            .expect("Failed to execute command");

        children.push(child);
    }

    children
}

fn should_ignore(path: &Path, ignore_patterns: &Option<Vec<String>>) -> bool {
    if let Some(patterns) = ignore_patterns {
        for pattern in patterns {
            let pattern_path = PathBuf::from_str(pattern).expect("Invalid pattern");
            if path.starts_with(&pattern_path) || path.to_str().unwrap_or("").contains(pattern) {
                return true;
            }
        }
    }
    false
}

fn force_kill_processes(port: u16) {
    // Kill any process using the port
    let _ = Command::new("sh")
        .arg("-c")
        .arg(format!("lsof -t -i:{} | xargs -r kill -9", port))
        .output();
    
    // Small delay to ensure processes are killed
    thread::sleep(Duration::from_millis(500));
}

fn is_port_available(port: u16) -> bool {
    let addr = format!("127.0.0.1:{}", port);
    std::net::TcpListener::bind(addr).is_ok()
}

fn restart_processes(
    children: &mut Vec<Child>,
    commands: &[String],
    env: &HashMap<String, String>,
    port: u16,
) {
    // Force kill all processes on the port
    force_kill_processes(port);
    
    // Kill existing child processes
    for child in children.iter_mut() {
        let _ = child.kill();
    }
    children.clear();
    
    // Ensure port is available
    if !is_port_available(port) {
        println!("WARNING: Port {} still in use after force kill", port);
        // Try one more time
        force_kill_processes(port);
    }
    
    // Start new processes
    println!("Starting new processes...");
    *children = execute_commands(commands, env);
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
    let port = config
        .env
        .get("PORT")
        .unwrap_or(&"8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);

    let last_changed = Arc::new(Mutex::new(Instant::now()));
    let is_restarting = Arc::new(Mutex::new(false));

    for res in rx {
        match res {
            Ok(event) => {
                if event
                    .paths
                    .iter()
                    .any(|path| should_ignore(path, &config.ignore))
                {
                    continue;
                }

                let now = Instant::now();
                let mut last_changed_time = last_changed.lock().unwrap();
                let mut is_restarting_flag = is_restarting.lock().unwrap();

                if !*is_restarting_flag && now.duration_since(*last_changed_time) > Duration::from_secs(2) {
                    println!("Changes detected in files: {:?}", event.paths);
                    *is_restarting_flag = true;
                    *last_changed_time = now;

                    restart_processes(&mut children, &config.commands, &config.env, port);

                    let is_restarting_clone = Arc::clone(&is_restarting);
                    thread::spawn(move || {
                        thread::sleep(Duration::from_secs(2));
                        let mut flag = is_restarting_clone.lock().unwrap();
                        *flag = false;
                    });
                } else {
                    println!("Skipping reload - debouncing active or restart in progress");
                }
            }
            Err(e) => println!("Watch error: {:?}", e),
        }
    }

    Ok(())
}