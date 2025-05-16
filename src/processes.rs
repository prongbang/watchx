use log::{info, warn};
use std::collections::HashMap;
use std::io::Error as IoError;
use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use scopeguard;
use std::thread;
use std::time::Duration;

use crate::command;

static RESTART_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

fn graceful_shutdown(pid: &str) -> Result<(), IoError> {
    let os = std::env::consts::OS;
    match os {
        "linux" | "macos" => {
            // First try SIGTERM
            if let Ok(_) = Command::new("kill").arg("-15").arg(pid).output() {
                // Give it some time to shutdown gracefully
                thread::sleep(Duration::from_millis(500));

                // Check if process still exists
                if Command::new("ps").arg("-p").arg(pid).output().is_ok() {
                    // If still running, force kill with SIGKILL
                    Command::new("kill").arg("-9").arg(pid).output()?;
                }
            }
        }
        "windows" => {
            // First try graceful shutdown
            if let Ok(_) = Command::new("taskkill").args(&["/PID", pid]).output() {
                thread::sleep(Duration::from_millis(500));

                // Check if process still exists and force kill if necessary
                if Command::new("tasklist")
                    .args(&["/FI", &format!("PID eq {}", pid)])
                    .output()
                    .is_ok()
                {
                    Command::new("taskkill")
                        .args(&["/F", "/PID", pid])
                        .output()?;
                }
            }
        }
        _ => {
            warn!("Graceful shutdown not implemented for OS: {}", os);
        }
    }
    Ok(())
}

fn is_port_available(port: u16) -> bool {
    let addr = format!("127.0.0.1:{}", port);
    std::net::TcpListener::bind(addr).is_ok()
}

fn force_kill(port: u16) {
    let os = std::env::consts::OS;

    // Function to get PIDs using a port
    let get_pids = |port: u16| -> Vec<String> {
        match os {
            "linux" | "macos" => {
                if let Ok(output) = Command::new("sh")
                    .arg("-c")
                    .arg(format!("lsof -t -i:{}", port))
                    .output() {
                    String::from_utf8_lossy(&output.stdout)
                        .split_whitespace()
                        .map(String::from)
                        .collect()
                } else {
                    Vec::new()
                }
            }
            "windows" => {
                if let Ok(output) = Command::new("cmd")
                    .args(&["/C", &format!("for /f \"tokens=5\" %a in ('netstat -ano ^| findstr :{} ^| findstr LISTENING') do @echo %a", port)])
                    .output() {
                    String::from_utf8_lossy(&output.stdout)
                        .lines()
                        .filter(|pid| pid.trim().parse::<u32>().is_ok())
                        .map(String::from)
                        .collect()
                } else {
                    Vec::new()
                }
            }
            _ => Vec::new()
        }
    };

    // Get initial list of PIDs
    let pids = get_pids(port);
    if pids.is_empty() {
        warn!("No processes found on port {}", port);
        return;
    }

    // Try graceful shutdown first
    for pid in &pids {
        if let Err(e) = graceful_shutdown(pid) {
            warn!("Failed to gracefully shutdown process {}: {}", pid, e);
        } else {
            info!("Successfully terminated process {} on port {}", pid, port);
        }
    }

    // Wait for processes to terminate
    thread::sleep(Duration::from_secs(1));

    // Check if any processes are still running on the port
    if !is_port_available(port) {
        warn!("Port {} still in use after graceful shutdown", port);
    } else {
        info!("Port {} successfully freed", port);
    }
}

pub fn restart(
    children: &mut Vec<Child>,
    commands: &[String],
    env: &HashMap<String, String>,
    port: u16,
) {
    if RESTART_IN_PROGRESS.load(Ordering::SeqCst) {
        info!("Restart already in progress, skipping duplicate restart");
        return;
    }
    
    // Set restart flag and ensure it's reset even if we panic
    RESTART_IN_PROGRESS.store(true, Ordering::SeqCst);
    let _reset_guard = scopeguard::guard((), |_| {
        RESTART_IN_PROGRESS.store(false, Ordering::SeqCst);
    });

    // Force kill processes only once
    force_kill(port);

    // Kill existing child processes with timeout
    for child in children.iter_mut() {
        if let Err(e) = child.kill() {
            warn!("Failed to kill process: {}", e);
        }
        
        // Wait for process to exit with timeout
        let start = std::time::Instant::now();
        while child.try_wait().map(|s| s.is_none()).unwrap_or(false) {
            if start.elapsed() > std::time::Duration::from_secs(5) {
                warn!("Process kill timed out after 5 seconds");
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }
    }
    children.clear();

    // Ensure port is available with retries
    let mut retries = 3;
    while !is_port_available(port) && retries > 0 {
        warn!(
            "Port {} still in use, retrying kill... ({} attempts left)",
            port, retries
        );
        force_kill(port);
        thread::sleep(Duration::from_secs(1));
        retries -= 1;
    }

    if !is_port_available(port) {
        warn!("Could not free port {} after multiple attempts", port);
        // Give it one last chance after a longer wait
        thread::sleep(Duration::from_secs(3));
        force_kill(port);
    }

    // Start new processes with enhanced output handling
    info!("Restarting...");
    *children = command::execute(commands, env);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_available() {
        assert!(is_port_available(0)); // Port 0 tells OS to assign random port
    }
}
