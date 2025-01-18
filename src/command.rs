use clap::{Parser, Subcommand};
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
        let (cmd, args) = parts.split_first().expect("Invalid command");

        let child = Command::new(cmd)
            .args(args)
            .envs(env.clone())
            .spawn()
            .expect("Failed to execute command");

        children.push(child);
    }

    children
}