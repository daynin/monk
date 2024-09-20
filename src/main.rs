use clap::Parser;
use monk::{install_hooks, read_config, run_hook, Cli, Commands};
use std::path::Path;

pub fn main() {
    let cli = Cli::parse();

    if !Path::new(".git").exists() {
        eprintln!("Error: .git directory not found. Ensure you're in a Git repository.");
        std::process::exit(1);
    }
    let config = read_config();

    match cli.command {
        Commands::Install => {
            install_hooks(&config);
        }
        Commands::Run { hook_name } => {
            run_hook(&config, &hook_name);
        }
    }
}
