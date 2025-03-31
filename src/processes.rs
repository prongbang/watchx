use std::process::{Child, Command};
use std::thread;
use std::time::Duration;
use std::collections::HashMap;
use log::{warn, info};

use crate::command;

fn force_kill(port: u16) {
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

pub fn restart(
    children: &mut Vec<Child>,
    commands: &[String],
    env: &HashMap<String, String>,
    port: u16,
) {
    // Force kill all processes on the port
    force_kill(port);
    
    // Kill existing child processes
    for child in children.iter_mut() {
        let _ = child.kill();
    }
    children.clear();
    
    // Ensure port is available
    if !is_port_available(port) {
        warn!("Port {} still in use after force kill", port);
        // Try one more time
        force_kill(port);
    }
    
    // Start new processes
    info!("Restarting...");
    *children = command::execute(commands, env);
}
