use glob::Pattern;
use log::{error, info, warn};
use notify::event::EventKind;
use notify::{RecommendedWatcher, RecursiveMode, Result, Watcher};
use regex::Regex;
use std::collections::HashSet;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, RecvTimeoutError};
use std::time::{Duration, Instant};

use crate::{command, config, icon, processes};

const DEBOUNCE_TIME: Duration = Duration::from_millis(250);
const WATCH_POLL_INTERVAL: Duration = Duration::from_millis(250);

enum IgnorePattern {
    Regex(Regex),
    Glob {
        pattern: Pattern,
        raw: String,
        basename_only: bool,
        directory_prefix: Option<DirectoryPrefix>,
    },
}

impl IgnorePattern {
    fn new(raw: &str) -> Option<Self> {
        if raw.starts_with('/') && raw.ends_with('/') && raw.len() > 2 {
            Regex::new(&raw[1..raw.len() - 1])
                .map(Self::Regex)
                .map_err(|e| warn!("Invalid ignore regex {}: {}", raw, e))
                .ok()
        } else {
            let normalized = normalize_path_str(raw);
            Pattern::new(&normalized)
                .map(|pattern| Self::Glob {
                    pattern,
                    basename_only: !normalized.contains('/'),
                    directory_prefix: DirectoryPrefix::new(&normalized),
                    raw: normalized,
                })
                .map_err(|e| warn!("Invalid ignore glob {}: {}", raw, e))
                .ok()
        }
    }

    fn matches(&self, candidate: &PathCandidate<'_>) -> bool {
        match self {
            Self::Regex(regex) => {
                regex.is_match(candidate.normalized()) || regex.is_match(candidate.without_root())
            }
            Self::Glob {
                pattern,
                raw,
                basename_only,
                directory_prefix,
            } => {
                pattern.matches(candidate.normalized())
                    || pattern.matches(candidate.without_root())
                    || (*basename_only
                        && candidate
                            .file_name()
                            .is_some_and(|file_name| pattern.matches(file_name)))
                    || directory_prefix
                        .as_ref()
                        .is_some_and(|prefix| prefix.matches(candidate.normalized()))
                    || matches_parent(candidate.parents(), raw, pattern)
            }
        }
    }
}

struct DirectoryPrefix {
    value: String,
    nested_marker: String,
}

impl DirectoryPrefix {
    fn new(pattern: &str) -> Option<Self> {
        let directory = pattern.trim_end_matches('/');
        if directory.is_empty() || !pattern.ends_with('/') || has_glob_meta(directory) {
            None
        } else {
            Some(Self {
                value: directory.to_string(),
                nested_marker: format!("/{directory}/"),
            })
        }
    }

    fn matches(&self, path: &str) -> bool {
        path == self.value
            || path
                .strip_prefix(&self.value)
                .is_some_and(|rest| rest.starts_with('/'))
            || path.contains(&self.nested_marker)
    }
}

struct NormalizedPath {
    value: String,
    without_root_start: usize,
}

impl NormalizedPath {
    fn new(path: &Path) -> Self {
        Self::from_string(normalize_path(path))
    }

    fn from_string(value: String) -> Self {
        let without_root_start = value.len() - value.trim_start_matches('/').len();
        Self {
            value,
            without_root_start,
        }
    }

    fn as_str(&self) -> &str {
        &self.value
    }

    fn without_root(&self) -> &str {
        &self.value[self.without_root_start..]
    }
}

struct PathCandidate<'a> {
    path: &'a Path,
    normalized: NormalizedPath,
    parents: Vec<NormalizedPath>,
}

impl<'a> PathCandidate<'a> {
    fn new(path: &'a Path) -> Self {
        let normalized = NormalizedPath::new(path);
        let parents = path.ancestors().skip(1).map(NormalizedPath::new).collect();

        Self {
            path,
            normalized,
            parents,
        }
    }

    fn normalized(&self) -> &str {
        self.normalized.as_str()
    }

    fn without_root(&self) -> &str {
        self.normalized.without_root()
    }

    fn file_name(&self) -> Option<&str> {
        self.path
            .file_name()
            .and_then(|file_name| file_name.to_str())
    }

    fn parents(&self) -> &[NormalizedPath] {
        &self.parents
    }
}

struct IgnoreMatcher {
    patterns: Vec<IgnorePattern>,
}

impl IgnoreMatcher {
    fn new(ignore_patterns: Option<&[String]>) -> Self {
        let patterns = ignore_patterns
            .unwrap_or(&[])
            .iter()
            .filter_map(|pattern| IgnorePattern::new(pattern))
            .collect();

        Self { patterns }
    }

    fn is_ignored(&self, path: &Path) -> bool {
        if path
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .is_some_and(|file_name| file_name.ends_with('~'))
        {
            return true;
        }

        let candidate = PathCandidate::new(path);

        self.patterns
            .iter()
            .any(|pattern| pattern.matches(&candidate))
    }
}

fn normalize_path(path: &Path) -> String {
    normalize_path_str(&path.to_string_lossy())
}

fn normalize_path_str(path: &str) -> String {
    path.replace('\\', "/").trim_start_matches("./").to_string()
}

fn has_glob_meta(pattern: &str) -> bool {
    pattern.contains(['*', '?', '[', ']'])
}

fn matches_parent(parents: &[NormalizedPath], raw: &str, pattern: &Pattern) -> bool {
    parents.iter().any(|parent| {
        pattern.matches(parent.as_str())
            || pattern.matches(parent.without_root())
            || (raw.ends_with('/')
                && (pattern.matches(&format!("{}/", parent.as_str()))
                    || pattern.matches(&format!("{}/", parent.without_root()))))
    })
}

#[cfg(test)]
fn should_ignore(path: &Path, ignore_patterns: &Option<Vec<String>>) -> bool {
    IgnoreMatcher::new(ignore_patterns.as_deref()).is_ignored(path)
}

fn make_clickable(path: &Path, current_dir: &Path, root_name: &str) -> String {
    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        current_dir.join(path)
    };

    // Get relative path from current directory
    let relative_path = absolute_path
        .strip_prefix(current_dir)
        .unwrap_or(path)
        .to_str()
        .unwrap_or_else(|| path.to_str().unwrap_or(""));

    let icon = icon::get_file_icon(path);

    format!(
        "\x1b]8;;file://{}\x1b\\{} {}/{}\x1b]8;;\x1b\\",
        absolute_path.display(),
        icon,
        root_name,
        relative_path
    )
}

fn collect_changed_paths(
    event: notify::Event,
    ignore_matcher: &IgnoreMatcher,
    pending_paths: &mut HashSet<PathBuf>,
) -> bool {
    if matches!(event.kind, EventKind::Access(_)) {
        return false;
    }

    event
        .paths
        .into_iter()
        .filter(|path| !ignore_matcher.is_ignored(path))
        .fold(false, |changed, path| pending_paths.insert(path) || changed)
}

fn log_changed_paths(paths: &HashSet<PathBuf>, current_dir: &Path, root_name: &str) {
    info!("Changed:");
    let mut paths = paths.iter().collect::<Vec<_>>();
    paths.sort_unstable();

    for path in paths {
        info!("{}", make_clickable(path, current_dir, root_name));
    }
}

fn configured_port(env: &std::collections::HashMap<String, String>) -> Option<u16> {
    env.get("PORT").and_then(|port| match port.parse::<u16>() {
        Ok(0) => None,
        Ok(port) => Some(port),
        Err(e) => {
            warn!("Ignoring invalid PORT value {}: {}", port, e);
            None
        }
    })
}

pub fn run(config_path: &str) -> Result<()> {
    info!("Config: {}", config_path);

    // Load configuration
    let config = config::read_config(config_path);

    // Channel to receive file change events
    let (tx, rx) = channel();

    // Set up file watcher with optimized polling
    let watch_config = notify::Config::default()
        .with_poll_interval(WATCH_POLL_INTERVAL)
        .with_compare_contents(false);
    let mut watcher: RecommendedWatcher = Watcher::new(tx, watch_config)?;
    let path = Path::new(&config.watch_dir);
    watcher.watch(path, RecursiveMode::Recursive)?;

    // Execute initial commands
    let mut children = command::execute(&config.commands, &config.env);
    let port = configured_port(&config.env);

    let ignore_matcher = IgnoreMatcher::new(config.ignore.as_deref());
    let current_dir = env::current_dir().unwrap_or_default();
    let root_name = current_dir
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .unwrap_or(".")
        .to_string();
    let mut pending_paths = HashSet::new();
    let mut pending_deadline: Option<Instant> = None;

    info!("Watch: {}", config.watch_dir);
    info!("Hot reload: {}ms", DEBOUNCE_TIME.as_millis());

    loop {
        if pending_deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            log_changed_paths(&pending_paths, &current_dir, &root_name);
            pending_paths.clear();
            pending_deadline = None;
            processes::restart(&mut children, &config.commands, &config.env, port);
            continue;
        }

        let result = if let Some(deadline) = pending_deadline {
            rx.recv_timeout(deadline.saturating_duration_since(Instant::now()))
        } else {
            rx.recv().map_err(|_| RecvTimeoutError::Disconnected)
        };

        match result {
            Ok(Ok(event)) => {
                if collect_changed_paths(event, &ignore_matcher, &mut pending_paths) {
                    pending_deadline = Some(Instant::now() + DEBOUNCE_TIME);
                }
            }
            Ok(Err(e)) => {
                error!("Watch error: {:?}", e);
            }
            Err(RecvTimeoutError::Timeout) => {
                if !pending_paths.is_empty() {
                    log_changed_paths(&pending_paths, &current_dir, &root_name);
                    pending_paths.clear();
                    pending_deadline = None;
                    processes::restart(&mut children, &config.commands, &config.env, port);
                }
            }
            Err(RecvTimeoutError::Disconnected) => {
                error!("Watch channel disconnected");
                break;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_should_ignore_tilde_files() {
        let path = PathBuf::from("test.txt~");
        assert!(should_ignore(&path, &Some(vec![])));
    }

    #[test]
    fn test_should_ignore_glob_patterns() {
        let patterns = Some(vec![
            String::from("*.log"),
            String::from("**/target/**"),
            String::from("**/node_modules/**"),
        ]);

        // Test file patterns
        assert!(should_ignore(&PathBuf::from("test/test.log"), &patterns));
        assert!(should_ignore(&PathBuf::from("/target/test.go"), &patterns));
        assert!(should_ignore(
            &PathBuf::from("test/target/test.go"),
            &patterns
        ));
        assert!(should_ignore(
            &PathBuf::from("test/target/sub/test.go"),
            &patterns
        ));
        assert!(should_ignore(
            &PathBuf::from("/node_modules/test.go"),
            &patterns
        ));
        assert!(should_ignore(
            &PathBuf::from("test/node_modules/test.go"),
            &patterns
        ));
        assert!(should_ignore(
            &PathBuf::from("test/node_modules/sub/test.go"),
            &patterns
        ));
        assert!(!should_ignore(&PathBuf::from("test/test.txt"), &patterns));
        assert!(!should_ignore(&PathBuf::from("test/test.go"), &patterns));
    }

    #[test]
    fn test_should_ignore_regex_patterns() {
        let patterns = Some(vec![
            String::from("/^test_.*\\.rs$/"),
            String::from("/.*_test\\.go$/"),
            String::from("/\\.git/"),
        ]);

        // Test file patterns
        assert!(should_ignore(&PathBuf::from("test_watcher.rs"), &patterns));
        assert!(should_ignore(&PathBuf::from("watcher_test.go"), &patterns));
        assert!(!should_ignore(&PathBuf::from("watcher.rs"), &patterns));

        // Test directory patterns
        assert!(should_ignore(&PathBuf::from(".git"), &patterns));
        assert!(!should_ignore(&PathBuf::from("src"), &patterns));
    }

    #[test]
    fn test_should_ignore_parent_directories() {
        let patterns = Some(vec![
            String::from("node_modules/"),
            String::from("/\\.git/"),
        ]);

        // Test nested files in ignored directories
        assert!(should_ignore(
            &PathBuf::from("node_modules/package.json"),
            &patterns
        ));
        assert!(should_ignore(&PathBuf::from(".git/config"), &patterns));
        assert!(!should_ignore(&PathBuf::from("src/main.rs"), &patterns));
    }
}
