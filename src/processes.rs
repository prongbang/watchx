use std::process::{Child, Command};
use std::thread;
use std::time::Duration;
use std::collections::HashMap;
use log::{warn, info};

use crate::command;

fn force_kill(port: u16) {
    // Detect operating system
    let os = std::env::consts::OS;
    
    match os {
        "linux" | "macos" => {
            // Use lsof on Unix-like systems (Linux, macOS)
            let output = Command::new("sh")
                .arg("-c")
                .arg(format!("lsof -t -i:{} | xargs -r kill -9", port))
                .output();
                
            match output {
                Ok(output) => {
                    if !output.stdout.is_empty() {
                        info!("Killed processes on port {}: {:?}", 
                              port, 
                              String::from_utf8_lossy(&output.stdout).trim());
                    }
                    if !output.stderr.is_empty() {
                        warn!("Error while killing processes: {:?}", 
                              String::from_utf8_lossy(&output.stderr).trim());
                    }
                },
                Err(e) => warn!("Failed to execute kill command: {}", e)
            }
        },
        "windows" => {
            // Use netstat and taskkill on Windows
            // First, find the PID
            let find_pid = Command::new("cmd")
                .args(&["/C", &format!("netstat -ano | findstr :{}", port)])
                .output();
                
            if let Ok(output) = find_pid {
                let output_str = String::from_utf8_lossy(&output.stdout);
                // Extract PIDs from netstat output
                for line in output_str.lines() {
                    if line.contains(&format!(":{}", port)) {
                        // The PID is usually the last column in netstat output
                        if let Some(pid) = line.split_whitespace().last() {
                            if let Ok(_pid_num) = pid.parse::<u32>() {
                                // Kill the process
                                let kill_result = Command::new("taskkill")
                                    .args(&["/F", "/PID", pid])
                                    .output();
                                    
                                match kill_result {
                                    Ok(kill_output) => {
                                        info!("Killed process {}: {:?}", 
                                              pid, 
                                              String::from_utf8_lossy(&kill_output.stdout).trim());
                                    },
                                    Err(e) => warn!("Failed to kill process {}: {}", pid, e)
                                }
                            }
                        }
                    }
                }
            }
        },
        _ => {
            warn!("Force kill not implemented for OS: {}", os);
        }
    }
    
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
