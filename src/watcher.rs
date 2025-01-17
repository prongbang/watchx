use notify::{RecommendedWatcher, RecursiveMode, Result, Watcher};
use std::sync::{mpsc::channel, Arc, Mutex};
use std::time::{Duration, Instant};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::thread;

use crate::{command, config, processes};

pub fn should_ignore(path: &Path, ignore_patterns: &Option<Vec<String>>) -> bool {
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

pub fn run(config_path: &str) -> Result<()> {
    println!("Starting ReloadX with config: {}", config_path);
    
    // Load configuration
    let config = config::read_config(config_path);

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
    let mut children = command::execute(&config.commands, &config.env);
    let port = config
        .env
        .get("PORT")
        .unwrap_or(&"8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);

    let last_changed = Arc::new(Mutex::new(Instant::now()));
    let is_restarting = Arc::new(Mutex::new(false));

    println!("👀 Watching for changes in {}", config.watch_dir);

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
                    println!("🔄 Changes detected in files: {:?}", event.paths);
                    *is_restarting_flag = true;
                    *last_changed_time = now;

                    processes::restart(&mut children, &config.commands, &config.env, port);

                    let is_restarting_clone = Arc::clone(&is_restarting);
                    thread::spawn(move || {
                        thread::sleep(Duration::from_secs(2));
                        let mut flag = is_restarting_clone.lock().unwrap();
                        *flag = false;
                    });
                } else {
                    println!("⏳ Skipping reload - debouncing active or restart in progress");
                }
            }
            Err(e) => println!("⚠️ Watch error: {:?}", e),
        }
    }

    Ok(())
}
