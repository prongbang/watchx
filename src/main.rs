use clap::Parser;
use notify::Result;

mod command;
mod config;
mod watcher;
mod processes;

fn main() -> Result<()> {
    let cli = command::Cli::parse();

    match cli.command {
        command::Commands::Run { config } => watcher::run(&config)?,
    }

    Ok(())
}