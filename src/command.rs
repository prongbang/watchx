use clap::{Parser, Subcommand};
use log::{info, warn};
use std::collections::HashMap;
use std::process::{Child, Command};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run the application with hot reloading
    Run {
        /// Optional path to config file
        #[arg(short, long, default_value = "watchx.yaml")]
        config: String,
    },
}

// Execute a list of commands in sequence
pub fn execute(commands: &[String], env: &HashMap<String, String>) -> Vec<Child> {
    let mut children = Vec::new();

    for command in commands {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if let Some((program, args)) = parts.split_first() {
            match Command::new(program).args(args).envs(env.clone()).spawn() {
                Ok(child) => {
                    info!("Started process {} (PID: {})", program, child.id());
                    children.push(child);
                }
                Err(e) => {
                    warn!("Failed to start process {}: {}", program, e);
                }
            }
        } else {
            warn!("Invalid command: {}", command);
        }
    }

    children
}
