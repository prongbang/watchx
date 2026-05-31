use clap::Parser;
use colored::*;
use env_logger::Builder;
use log::{Level, LevelFilter};
use notify::Result;

mod command;
mod config;
mod icon;
mod processes;
mod watcher;

fn main() -> Result<()> {
    // Initialize logger with custom format and colors
    Builder::new()
        .filter_level(LevelFilter::Info)
        .format(|buf, record| {
            use std::io::Write;
            let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
            let level = match record.level() {
                Level::Error => "ERROR".red().bold(),
                Level::Warn => "WARN".yellow().bold(),
                Level::Info => "INFO".green().bold(),
                Level::Debug => "DEBUG".blue().bold(),
                Level::Trace => "TRACE".normal().bold(),
            };
            let message = record.args().to_string();
            let message = if message.starts_with("  ") {
                // Indented messages (file paths) should be dimmed
                message.dimmed()
            } else {
                message.normal()
            };
            writeln!(buf, "{} {} {}", timestamp.dimmed(), level, message)
        })
        .init();

    let cli = command::Cli::parse();

    match cli.command {
        command::Commands::Init {
            config: config_path,
            force,
        } => {
            if let Err(e) = config::init_config(&config_path, force) {
                log::error!("Failed to initialize configuration file: {}", e);
                std::process::exit(1);
            }

            log::info!("Created config: {}", config_path);
        }
        command::Commands::Run { config } => watcher::run(&config)?,
    }

    Ok(())
}
