use log::{info, warn};
use std::collections::HashMap;
use std::io::Error as IoError;
use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use crate::command;

static RESTART_IN_PROGRESS: AtomicBool = AtomicBool::new(false);
const CHILD_TERM_TIMEOUT: Duration = Duration::from_millis(1500);
const CHILD_KILL_TIMEOUT: Duration = Duration::from_millis(500);
const CHILD_WAIT_INTERVAL: Duration = Duration::from_millis(25);
const PID_TERM_TIMEOUT: Duration = Duration::from_millis(500);
const PORT_RELEASE_TIMEOUT: Duration = Duration::from_millis(750);
const PORT_WAIT_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Clone, Copy)]
enum Signal {
    Term,
    Kill,
}

fn graceful_shutdown(pid: &str) -> Result<(), IoError> {
    let os = std::env::consts::OS;
    match os {
        "linux" | "macos" => {
            send_pid_signal(pid, Signal::Term)?;
            if !wait_until_pid_exits(pid, PID_TERM_TIMEOUT) {
                send_pid_signal(pid, Signal::Kill)?;
            }
        }
        "windows" => {
            send_pid_signal(pid, Signal::Term)?;
            if !wait_until_pid_exits(pid, PID_TERM_TIMEOUT) {
                send_pid_signal(pid, Signal::Kill)?;
            }
        }
        _ => {
            warn!("Graceful shutdown not implemented for OS: {}", os);
        }
    }
    Ok(())
}

fn send_pid_signal(pid: &str, signal: Signal) -> Result<(), IoError> {
    match std::env::consts::OS {
        "linux" | "macos" => {
            let signal = match signal {
                Signal::Term => "-15",
                Signal::Kill => "-9",
            };
            Command::new("kill").arg(signal).arg(pid).output()?;
        }
        "windows" => match signal {
            Signal::Term => {
                Command::new("taskkill").args(["/PID", pid]).output()?;
            }
            Signal::Kill => {
                Command::new("taskkill")
                    .args(["/F", "/PID", pid])
                    .output()?;
            }
        },
        _ => {}
    }

    Ok(())
}

fn process_exists(pid: &str) -> bool {
    match std::env::consts::OS {
        "linux" | "macos" => Command::new("ps")
            .arg("-p")
            .arg(pid)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false),
        "windows" => Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid)])
            .output()
            .map(|output| {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout
                    .lines()
                    .any(|line| line.split_whitespace().any(|part| part == pid))
            })
            .unwrap_or(false),
        _ => false,
    }
}

fn wait_until_pid_exits(pid: &str, timeout: Duration) -> bool {
    let start = Instant::now();
    loop {
        if !process_exists(pid) {
            return true;
        }

        if start.elapsed() >= timeout {
            return false;
        }

        thread::sleep(CHILD_WAIT_INTERVAL);
    }
}

fn is_port_available(port: u16) -> bool {
    let addr = format!("127.0.0.1:{}", port);
    std::net::TcpListener::bind(addr).is_ok()
}

fn wait_until_port_available(port: u16, timeout: Duration) -> bool {
    let start = Instant::now();
    loop {
        if is_port_available(port) {
            return true;
        }

        if start.elapsed() >= timeout {
            return false;
        }

        thread::sleep(PORT_WAIT_INTERVAL);
    }
}

fn pids_on_port(port: u16) -> Vec<String> {
    let os = std::env::consts::OS;

    match os {
        "linux" | "macos" => {
            let port_filter = format!("TCP:{}", port);
            Command::new("lsof")
                .args(["-ti", &port_filter, "-sTCP:LISTEN"])
                .output()
                .map(|output| {
                    String::from_utf8_lossy(&output.stdout)
                        .split_whitespace()
                        .map(String::from)
                        .collect()
                })
                .unwrap_or_default()
        }
        "windows" => {
            let netstat_command = format!(
                "for /f \"tokens=5\" %a in ('netstat -ano ^| findstr :{} ^| findstr LISTENING') do @echo %a",
                port
            );
            if let Ok(output) = Command::new("cmd").args(["/C", &netstat_command]).output() {
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .filter(|pid| pid.trim().parse::<u32>().is_ok())
                    .map(String::from)
                    .collect()
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    }
}

fn force_kill(port: u16) {
    // Get initial list of PIDs
    let pids = pids_on_port(port);
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

    // Check if any processes are still running on the port
    if !wait_until_port_available(port, PORT_RELEASE_TIMEOUT) {
        warn!("Port {} still in use after graceful shutdown", port);
    } else {
        info!("Port {} successfully freed", port);
    }
}

fn stop_child(child: &mut Child) {
    match child.try_wait() {
        Ok(Some(_)) => return,
        Ok(None) => {}
        Err(e) => {
            warn!("Failed to check process status: {}", e);
            return;
        }
    }

    if let Err(e) = signal_child_group(child, Signal::Term) {
        warn!("Failed to terminate process {} group: {}", child.id(), e);

        if let Err(kill_error) = child.kill() {
            warn!(
                "Failed to kill process {} directly: {}",
                child.id(),
                kill_error
            );
            return;
        }
    }

    if wait_for_child(child, CHILD_TERM_TIMEOUT) {
        return;
    }

    warn!(
        "Process {} did not exit after TERM, forcing kill",
        child.id()
    );

    if let Err(e) = signal_child_group(child, Signal::Kill) {
        warn!("Failed to kill process {} group: {}", child.id(), e);

        if let Err(kill_error) = child.kill() {
            warn!(
                "Failed to kill process {} directly: {}",
                child.id(),
                kill_error
            );
            return;
        }
    }

    if !wait_for_child(child, CHILD_KILL_TIMEOUT) {
        warn!("Process {} kill timed out", child.id());
    }
}

fn wait_for_child(child: &mut Child, timeout: Duration) -> bool {
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return true,
            Ok(None) if start.elapsed() < timeout => thread::sleep(CHILD_WAIT_INTERVAL),
            Ok(None) => return false,
            Err(e) => {
                warn!("Failed while waiting for process {}: {}", child.id(), e);
                return true;
            }
        }
    }
}

fn signal_child_group(child: &mut Child, signal: Signal) -> Result<(), IoError> {
    #[cfg(unix)]
    {
        let pid = child.id().to_string();
        let signal = match signal {
            Signal::Term => "-TERM",
            Signal::Kill => "-KILL",
        };
        Command::new("kill")
            .args([signal, "--", &format!("-{}", pid)])
            .status()?;
        Ok(())
    }

    #[cfg(windows)]
    {
        let pid = child.id().to_string();
        let mut command = Command::new("taskkill");
        command.arg("/T");
        if matches!(signal, Signal::Kill) {
            command.arg("/F");
        }
        command.args(["/PID", &pid]).status()?;
        Ok(())
    }

    #[cfg(not(any(unix, windows)))]
    {
        child.kill()
    }
}

pub fn restart(
    children: &mut Vec<Child>,
    commands: &[String],
    env: &HashMap<String, String>,
    port: Option<u16>,
) {
    if RESTART_IN_PROGRESS
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        info!("Restart already in progress, skipping duplicate restart");
        return;
    }

    // Set restart flag and ensure it's reset even if we panic
    let _reset_guard = scopeguard::guard((), |_| {
        RESTART_IN_PROGRESS.store(false, Ordering::Release);
    });

    for child in children.iter_mut() {
        stop_child(child);
    }
    children.clear();

    if let Some(port) = port {
        if !is_port_available(port) {
            warn!("Port {} still in use after stopping child processes", port);
            force_kill(port);
        }
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
        let Ok(listener) = std::net::TcpListener::bind("127.0.0.1:0") else {
            return;
        };
        let port = listener.local_addr().unwrap().port();

        assert!(!is_port_available(port));
        drop(listener);
        assert!(is_port_available(port));
    }
}
