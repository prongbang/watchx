use notify::{RecommendedWatcher, RecursiveMode, Result, Watcher};
use std::sync::{mpsc::channel, Arc, Mutex};
use std::time::{Duration, Instant};
use std::path::Path;
use std::thread;
use glob::Pattern;
use log::{info, warn, error};
use std::env;

use crate::{command, config, processes};

pub fn should_ignore(path: &Path, ignore_patterns: &Option<Vec<String>>) -> bool {
    if let Some(patterns) = ignore_patterns {
        let path_str = path.to_str().unwrap_or("");
        for pattern in patterns {
            if let Ok(glob_pattern) = Pattern::new(pattern) {
                if glob_pattern.matches(path_str) {
                    return true;
                }
            }
        }
    }
    false
}

fn get_file_icon(path: &Path) -> &'static str {
    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    
    match extension {
        "rs" => "🦀",  // Rust
        "js" | "jsx" | "ts" | "tsx" => "📜",  // JavaScript/TypeScript
        "py" => "🐍",  // Python
        "go" => "🐹",  // Go
        "java" => "☕",  // Java
        "c" | "cpp" | "h" | "hpp" => "⚡",  // C/C++
        "html" | "htm" => "🌐",  // HTML
        "css" | "scss" | "sass" => "🎨",  // CSS
        "json" => "📋",  // JSON
        "md" => "📝",  // Markdown
        "yml" | "yaml" => "⚙️",  // YAML
        "toml" => "📦",  // TOML
        "sh" | "bash" => "🐚",  // Shell
        "sql" => "💾",  // SQL
        "git" | "gitignore" => "🔧",  // Git
        "lock" => "🔒",  // Lock files
        "log" => "📊",  // Log files
        "txt" => "📄",  // Text files
        "pdf" => "📕",  // PDF
        "jpg" | "jpeg" | "png" | "gif" | "svg" => "🖼️",  // Images
        "mp4" | "mov" | "avi" => "🎥",  // Videos
        "mp3" | "wav" | "ogg" => "🎵",  // Audio
        "zip" | "tar" | "gz" => "📦",  // Archives
        _ => "📄",  // Default
    }
}

fn make_clickable(path: &Path) -> String {
    let path_str = path.to_str().unwrap_or("");
    let canonical_path = path.canonicalize().unwrap_or(path.to_path_buf());
    let current_dir = env::current_dir().unwrap_or_default();
    
    // Get relative path from current directory
    let relative_path = path.strip_prefix(&current_dir)
        .unwrap_or(path)
        .to_str()
        .unwrap_or(path_str);
    
    let icon = get_file_icon(path);
    
    format!("\x1b]8;;file://{}\x1b\\{} {}/{}\x1b]8;;\x1b\\", 
        canonical_path.display(), 
        icon,
        current_dir.file_name().unwrap_or_default().to_str().unwrap_or("."),
        relative_path
    )
}

pub fn run(config_path: &str) -> Result<()> {
    info!("Config: {}", config_path);
    
    // Load configuration
    let config = config::read_config(config_path);

    // Channel to receive file change events
    let (tx, rx) = channel();

    // Set up file watcher with optimized polling
    let watch_config = notify::Config::default()
        .with_poll_interval(Duration::from_secs(1))
        .with_compare_contents(false);
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

    // Use atomic types for better performance
    let last_changed = Arc::new(Mutex::new(Instant::now()));
    let is_restarting = Arc::new(Mutex::new(false));
    let debounce_time = Duration::from_secs(1);

    info!("Watch: {}", config.watch_dir);
    info!("Hot reload: 1s");

    // Spawn a single thread for debouncing
    let is_restarting_clone = Arc::clone(&is_restarting);
    thread::spawn(move || {
        loop {
            thread::sleep(debounce_time);
            if let Ok(mut flag) = is_restarting_clone.lock() {
                *flag = false;
            }
        }
    });

    let mut last_warn_time = Instant::now();

    // Use a timeout for the receiver to prevent blocking indefinitely
    let timeout = Duration::from_millis(100);
    
    loop {
        // Use a timeout to prevent blocking indefinitely
        match rx.recv_timeout(timeout) {
            Ok(Ok(event)) => {
                // Check if any non-ignored files have changed
                let has_non_ignored_changes = event.paths.iter().any(|path| !should_ignore(path, &config.ignore));
                
                if !has_non_ignored_changes {
                    continue;
                }

                let now = Instant::now();
                let mut last_changed_time = last_changed.lock().unwrap();
                let mut is_restarting_flag = is_restarting.lock().unwrap();

                if !*is_restarting_flag && now.duration_since(*last_changed_time) > debounce_time {
                    info!("Changed:");
                    for path in &event.paths {
                        if !should_ignore(path, &config.ignore) {
                            info!("  {}", make_clickable(path));
                        }
                    }
                    *is_restarting_flag = true;
                    *last_changed_time = now;
                    last_warn_time = now; // Reset warning time when changes are processed

                    processes::restart(&mut children, &config.commands, &config.env, port);
                } else if now.duration_since(last_warn_time) > debounce_time {
                    warn!("Debounce: skipping reload");
                    last_warn_time = now;
                }
            }
            Ok(Err(e)) => {
                error!("Watch error: {:?}", e);
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // Timeout occurred, continue the loop
                continue;
            }
            Err(e) => {
                error!("Channel error: {:?}", e);
                // If the channel is closed, exit the loop
                break;
            }
        }
    }

    Ok(())
}
