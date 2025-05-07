use clap::{Parser, Subcommand};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(name = "monk")]
#[command(about = "A simple Git hooks manager written in Rust")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Install all hooks listed in monk.yaml")]
    Install,
    #[command(about = "Run specified hook")]
    Run { hook_name: String },
    #[command(about = "Uninstall all monk hooks and restore backups if available")]
    Uninstall,
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(flatten)]
    hooks: std::collections::HashMap<String, Hook>,
}

#[derive(Deserialize)]
pub struct Hook {
    commands: Vec<String>,
}

pub fn read_config() -> Config {
    let config_str = fs::read_to_string("monk.yaml").expect("Failed to read monk.yaml");
    serde_yaml::from_str(&config_str).expect("Failed to parse monk.yaml")
}

pub fn install_hooks(config: &Config) {
    let git_hooks_dir = ".git/hooks";
    if !Path::new(git_hooks_dir).exists() {
        fs::create_dir_all(git_hooks_dir).expect("Failed to create .git/hooks directory");
    }

    for hook_name in config.hooks.keys() {
        let hook_path = format!("{}/{}", git_hooks_dir, hook_name);
        let backup_path = format!("{}.backup", hook_path);

        if Path::new(&hook_path).exists() && !Path::new(&backup_path).exists() {
            fs::rename(&hook_path, &backup_path)
                .unwrap_or_else(|_| panic!("Failed to backup existing hook: {}", hook_name));
            println!("Backed up existing hook: {}", hook_name);
        }

        install_hook(hook_name);
    }
}

pub fn install_hook(hook_name: &str) {
    let git_hooks_dir = ".git/hooks";
    if !Path::new(git_hooks_dir).exists() {
        fs::create_dir_all(git_hooks_dir).expect("Failed to create .git/hooks directory");
    }

    let hook_path = format!("{}/{}", git_hooks_dir, hook_name);

    #[cfg(unix)]
    let hook_content = format!(
        "#!/bin/sh\n\
        if monk -h >/dev/null 2>&1\n\
        then\n\
            exec monk run {hook_name}\n\
        else\n\
            cargo install monk\n\
            exec monk run {hook_name}\n\
        fi"
    );

    #[cfg(windows)]
    let hook_content = format!(
        "@echo off\r\n\
        monk -h >nul 2>&1\r\n\
        if %errorlevel% == 0 (\r\n\
            monk run {hook_name}\r\n\
        ) else (\r\n\
            cargo install monk\r\n\
            monk run {hook_name}\r\n\
        )"
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

pub fn uninstall_hooks(config: &Config) {
    let git_hooks_dir = ".git/hooks";
    for hook_name in config.hooks.keys() {
        let hook_path = format!("{}/{}", git_hooks_dir, hook_name);
        let backup_path = format!("{}.backup", hook_path);

        if Path::new(&backup_path).exists() {
            fs::rename(&backup_path, &hook_path)
                .unwrap_or_else(|_| panic!("Failed to restore backup for {}", hook_name));
            println!("Restored backup for {}", hook_name);
        } else if Path::new(&hook_path).exists() {
            fs::remove_file(&hook_path)
                .unwrap_or_else(|_| panic!("Failed to remove hook: {}", hook_name));
            println!("Removed hook {}", hook_name);
        } else {
            println!("No hook or backup found for {}", hook_name);
        }
    }
}

pub fn run_hook(config: &Config, hook_name: &str) {
    if let Some(hook) = config.hooks.get(hook_name) {
        for command_str in &hook.commands {
            println!("Running command: {}", command_str);

            let mut command = std::process::Command::new(if cfg!(windows) { "cmd" } else { "sh" });

            if cfg!(windows) {
                command.args(["/C", command_str]);
            } else {
                command.args(["-c", command_str]);
            }

            let status = command.status().expect("Failed to execute command");
            if !status.success() {
                std::process::exit(status.code().unwrap_or(1));
            }
        }
    } else {
        eprintln!("No commands defined for hook '{}'", hook_name);
        std::process::exit(1);
    }
}

pub fn init() {
    let config = read_config();
    install_hooks(&config);
}
