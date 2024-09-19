use clap::{Parser, Subcommand};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(name = "monk")]
#[command(about = "A simple Git hooks manager written in Rust")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Install,
    Run {
        hook_name: String,
    },
}

#[derive(Deserialize)]
struct Config {
    #[serde(flatten)]
    hooks: std::collections::HashMap<String, Hook>,
}

#[derive(Deserialize)]
struct Hook {
    commands: Vec<String>,
}

fn main() {
    let cli = Cli::parse();

    if !Path::new(".git").exists() {
        eprintln!("Error: .git directory not found. Ensure you're in a Git repository.");
        std::process::exit(1);
    }

    match cli.command {
        Commands::Install => {
            let config = read_config();
            install_hooks(&config);
        }
        Commands::Run { hook_name } => {
            let config = read_config();
            run_hook(&config, &hook_name);
        }
    }
}

fn read_config() -> Config {
    let config_str = fs::read_to_string("monk.yaml")
        .expect("Failed to read monk.yaml");
    serde_yaml::from_str(&config_str).expect("Failed to parse monk.yaml")
}

fn install_hooks(config: &Config) {
    for (hook_name, _) in &config.hooks {
        install_hook(hook_name);
    }
}

fn install_hook(hook_name: &str) {
    let git_hooks_dir = ".git/hooks";
    if !Path::new(git_hooks_dir).exists() {
        fs::create_dir_all(git_hooks_dir).expect("Failed to create .git/hooks directory");
    }
    let hook_path = format!("{}/{}", git_hooks_dir, hook_name);
    let hook_content = format!(
        "#!/bin/sh\nexec monk run {} \"$@\"\n",
        hook_name
    );
    fs::write(&hook_path, hook_content)
        .unwrap_or_else(|_| panic!("Failed to write hook script to {}", hook_path));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&hook_path)
            .expect("Failed to get file permissions")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms).expect("Failed to set file permissions");
    }
}

fn run_hook(config: &Config, hook_name: &str) {
    if let Some(hook) = config.hooks.get(hook_name) {
        for command_str in &hook.commands {
            println!("Running command: {}", command_str);
            let status = std::process::Command::new("sh")
                .arg("-c")
                .arg(command_str)
                .status()
                .expect("Failed to execute command");
            if !status.success() {
                std::process::exit(status.code().unwrap_or(1));
            }
        }
    } else {
        eprintln!("No commands defined for hook '{}'", hook_name);
        std::process::exit(1);
    }
}