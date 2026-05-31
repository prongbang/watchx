use clap::{Parser, Subcommand};
use log::{info, warn};
use std::collections::HashMap;
use std::process::{Child, Command};

#[cfg(unix)]
use std::os::unix::process::CommandExt as _;
#[cfg(windows)]
use std::os::windows::process::CommandExt as _;

#[cfg(windows)]
const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a watchx configuration file
    Init {
        /// Optional path to config file
        #[arg(short, long, default_value = "watchx.yaml")]
        config: String,

        /// Overwrite an existing config file
        #[arg(short, long)]
        force: bool,
    },

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
            let mut cmd = Command::new(program);
            cmd.args(args).envs(env);

            #[cfg(unix)]
            cmd.process_group(0);

            #[cfg(windows)]
            cmd.creation_flags(CREATE_NEW_PROCESS_GROUP);

            match cmd.spawn() {
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
