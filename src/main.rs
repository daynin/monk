use clap::Parser;
use monk::{install_hooks, read_config, run_hook, uninstall_hooks, Cli, Commands};
use std::path::Path;
use colored::*;
use console::Emoji;

static CROSS: Emoji<'_, '_> = Emoji("❌ ", "✗ ");
static GEAR: Emoji<'_, '_> = Emoji("⚙️ ", "* ");

pub fn main() {
    let cli = Cli::parse();

    if !Path::new(".git").exists() {
        eprintln!("{} {}", CROSS, "Error: .git directory not found. Ensure you're in a Git repository.".red().bold());
        eprintln!("   {}", "Initialize a Git repository with: git init".yellow());
        std::process::exit(1);
    }
    
    let config = match read_config() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("{} {}", CROSS, "Failed to read monk.yaml configuration".red().bold());
            eprintln!("   {}", format!("Error: {}", e).yellow());
            eprintln!("   {}", "Create a monk.yaml file in your project root.".yellow());
            eprintln!("   {}", "Example configuration:".yellow());
            eprintln!("   {}", "pre-commit:".blue());
            eprintln!("   {}", "  commands:".blue());
            eprintln!("   {}", "    - cargo fmt -- --check".blue());
            eprintln!("   {}", "    - cargo clippy".blue());
            std::process::exit(1);
        }
    };

    println!("{} Monk Git hooks manager", GEAR);

    match cli.command {
        Commands::Install => install_hooks(&config),
        Commands::Run { hook_name } => run_hook(&config, &hook_name),
        Commands::Uninstall => uninstall_hooks(&config),
    }
}
