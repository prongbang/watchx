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
            // Find PIDs using port first
            let find_pid = Command::new("sh")
                .arg("-c")
                .arg(format!("lsof -t -i:{}", port))
                .output();
                
            if let Ok(output) = find_pid {
                let pids = String::from_utf8_lossy(&output.stdout);
                for pid in pids.split_whitespace() {
                    // Kill each process individually instead of using xargs
                    let kill_result = Command::new("kill")
                        .args(&["-9", pid])
                        .output();
                        
                    match kill_result {
                        Ok(kill_output) => {
                            info!("Killed process {} on port {}", pid, port);
                            if !kill_output.stderr.is_empty() {
                                warn!("Error killing process {}: {:?}", 
                                      pid, 
                                      String::from_utf8_lossy(&kill_output.stderr).trim());
                            }
                        },
                        Err(e) => warn!("Failed to kill process {}: {}", pid, e)
                    }
                }
            } else {
                warn!("Failed to find processes on port {}", port);
            }
        },
        "windows" => {
            // For Windows, use a more reliable method
            let find_pid = Command::new("cmd")
                .args(&["/C", &format!("for /f \"tokens=5\" %a in ('netstat -ano ^| findstr :{} ^| findstr LISTENING') do @echo %a", port)])
                .output();
                
            if let Ok(output) = find_pid {
                let output_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                for pid in output_str.lines() {
                    if let Ok(_) = pid.trim().parse::<u32>() {
                        // Kill the process
                        let kill_result = Command::new("taskkill")
                            .args(&["/F", "/PID", pid.trim()])
                            .output();
                            
                        match kill_result {
                            Ok(kill_output) => {
                                info!("Killed process {} on port {}", pid.trim(), port);
                                if !kill_output.stderr.is_empty() {
                                    warn!("Error killing process {}: {:?}", 
                                          pid.trim(), 
                                          String::from_utf8_lossy(&kill_output.stderr).trim());
                                }
                            },
                            Err(e) => warn!("Failed to kill process {}: {}", pid.trim(), e)
                        }
                    } else {
                        warn!("Invalid PID format: {}", pid);
                    }
                }
            } else {
                warn!("Failed to find processes on port {}", port);
            }
        },
        _ => {
            warn!("Force kill not implemented for OS: {}", os);
        }
    }
    
    // Increase the delay to ensure processes are fully terminated
    thread::sleep(Duration::from_millis(1000));
    
    // Double-check if port is available after killing
    if !is_port_available(port) {
        warn!("Port {} still in use after force kill attempt", port);
    } else {
        info!("Port {} successfully freed", port);
    }
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
    
    // Ensure port is available with retries
    let mut retries = 3;
    while !is_port_available(port) && retries > 0 {
        warn!("Port {} still in use, retrying kill... ({} attempts left)", port, retries);
        force_kill(port);
        thread::sleep(Duration::from_secs(1));
        retries -= 1;
    }
    
    if !is_port_available(port) {
        warn!("Could not free port {} after multiple attempts", port);
    }
    
    // Start new processes
    info!("Restarting...");
    *children = command::execute(commands, env);
}
